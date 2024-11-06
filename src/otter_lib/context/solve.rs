use rand::{seq::IteratorRandom, Rng};

use crate::{
    config::Config,
    context::{
        analysis::AnalysisResult,
        core::{ContextFailure, StepInfo},
        stores::LevelIndex,
        Context, Report, SolveStatus,
    },
    errors::StepErr,
    structures::{
        clause::Clause,
        literal::{Literal, LiteralSource},
        variable::{list::VariableList, VariableId, BCP::BCPIssue},
    },
};

impl Context {
    pub fn solve(&mut self) -> Result<Report, ContextFailure> {
        let this_total_time = std::time::Instant::now();

        match self.preprocess() {
            Ok(()) => {}
            Err(_) => panic!("Preprocessing failure"),
        };

        if self.clause_store.clause_count() == 0 {
            self.status = SolveStatus::NoClauses;
            return Ok(Report::Satisfiable);
        }

        if self.config.show_stats {
            if let Some(window) = &mut self.window {
                window.draw_window(&self.config);
            }
        }

        let config_clone = self.config.clone();
        let time_limit = config_clone.time_limit;

        'solve_loop: loop {
            self.counters.time = this_total_time.elapsed();
            if time_limit.is_some_and(|limit| self.counters.time > limit) {
                return Ok(self.report());
            }

            match self.step(&config_clone) {
                Ok(StepInfo::One) => continue 'solve_loop,
                Ok(StepInfo::ChoiceMade) => continue 'solve_loop,
                Ok(StepInfo::ChoicesExhausted) => break 'solve_loop Ok(self.report()),
                Ok(StepInfo::Conflict(_)) => break 'solve_loop Ok(self.report()),

                Err(StepErr::Backfall) => panic!("Backjumping failed"),
                Err(StepErr::AnalysisFailure) => panic!("Analysis failed"),
                Err(StepErr::ChoiceFailure) => panic!("Choice failure"),
                Err(e) => {
                    panic!("{e:?}");
                }
            }
        }
    }

    pub fn step(&mut self, config: &Config) -> Result<StepInfo, StepErr> {
        self.counters.iterations += 1;

        'search: while let Some((literal, _source, _)) = self.variables.get_consequence() {
            match self.BCP(literal) {
                Ok(()) => {}
                Err(BCPIssue::Conflict(key)) => {
                    let Ok(analysis_result) = self.conflict_analysis(key, config) else {
                        log::error!(target: crate::log::targets::STEP, "Conflict analysis failed.");
                        return Err(StepErr::AnalysisFailure);
                    };

                    match analysis_result {
                        AnalysisResult::FundamentalConflict(key) => {
                            self.status = SolveStatus::NoSolution(key);

                            return Ok(StepInfo::Conflict(key));
                        }

                        AnalysisResult::QueueConflict(key) => {
                            self.status = SolveStatus::NoSolution(key);

                            return Ok(StepInfo::Conflict(key));
                        }

                        AnalysisResult::Proof(key, literal) => {
                            self.status = SolveStatus::Proof(key);

                            self.backjump(0);

                            match self.q_literal(literal, LiteralSource::Resolution(key)) {
                                Ok(()) => {}
                                Err(_) => return Err(StepErr::QueueProof(key)),
                            }
                        }

                        AnalysisResult::MissedImplication(key, literal) => {
                            self.status = SolveStatus::MissedImplication(key);

                            let the_clause = self.clause_store.get(key)?;

                            match self.backjump_level(the_clause.literal_slice()) {
                                None => return Err(StepErr::Backfall),
                                Some(index) => self.backjump(index),
                            }

                            match self.q_literal(literal, LiteralSource::Missed(key)) {
                                Ok(()) => {}
                                Err(_) => return Err(StepErr::QueueConflict(key)),
                            };

                            continue 'search;
                        }

                        AnalysisResult::AssertingClause(key, literal) => {
                            self.status = SolveStatus::AssertingClause(key);

                            let the_clause = self.clause_store.get(key)?;

                            match self.backjump_level(the_clause.literal_slice()) {
                                None => return Err(StepErr::Backfall),
                                Some(index) => self.backjump(index),
                            }

                            match self.q_literal(literal, LiteralSource::Analysis(key)) {
                                Ok(()) => {}
                                Err(_) => return Err(StepErr::QueueConflict(key)),
                            }

                            self.conflict_ceremony(config)?;
                            return Ok(StepInfo::One);
                        }
                    }
                }
                Err(BCPIssue::CorruptWatch) => return Err(StepErr::BCPFailure),
            }
        }

        self.make_choice(config)
    }
}

impl Context {
    fn conflict_ceremony(&mut self, config: &Config) -> Result<(), StepErr> {
        self.counters.conflicts += 1;
        self.counters.conflicts_in_memory += 1;

        if self.counters.conflicts_in_memory
            % (config.luby_constant * self.counters.luby.current()) as usize
            == 0
        {
            self.counters.luby.next();
            if let Some(window) = &self.window {
                window.update_counters(&self.counters);
                window.flush();
            }

            if config.restarts_allowed {
                self.backjump(0);
                self.counters.restarts += 1;
                self.counters.conflicts_in_memory = 0;
            }

            if config.reduction_allowed
                && ((self.counters.restarts % config.reduction_interval) == 0)
            {
                log::debug!(target: crate::log::targets::REDUCTION, "Reduction after {} restarts", self.counters.restarts);
                self.clause_store.reduce(config)?;
            }
        }
        Ok(())
    }

    fn make_choice(&mut self, config: &Config) -> Result<StepInfo, StepErr> {
        match self.get_unassigned(config) {
            Some(choice_index) => {
                self.counters.decisions += 1;
                self.levels.get_fresh();

                log::trace!(target: crate::log::targets::STEP,
                    "Choice of {choice_index} at level {}",
                    self.levels.top().index(),
                );
                let choice_literal = {
                    let choice_id = choice_index as VariableId;
                    match self.variables.get_unsafe(choice_index).previous_value() {
                        Some(polarity) => Literal::new(choice_id, polarity),
                        None => {
                            let random_value = self.counters.rng.gen_bool(config.polarity_lean);
                            Literal::new(choice_id, random_value)
                        }
                    }
                };
                match self.q_literal(choice_literal, LiteralSource::Choice) {
                    Ok(()) => {}
                    Err(_) => return Err(StepErr::ChoiceFailure),
                };

                self.status = SolveStatus::ChoiceMade;
                Ok(StepInfo::ChoiceMade)
            }
            None => {
                self.status = SolveStatus::FullValuation;
                Ok(StepInfo::ChoicesExhausted)
            }
        }
    }

    fn get_unassigned(&mut self, config: &Config) -> Option<usize> {
        match self.counters.rng.gen_bool(config.random_choice_frequency) {
            true => self
                .variables
                .iter()
                .filter(|variable| variable.value().is_none())
                .choose(&mut self.counters.rng)
                .map(|variable| variable.index()),
            false => {
                while let Some(index) = self.variables.heap_pop_most_active() {
                    let the_variable = self.variables.get_unsafe(index);
                    if self.variables.value_at(the_variable.index()).is_none() {
                        return Some(the_variable.index());
                    }
                }
                self.variables
                    .iter()
                    .filter(|variable| variable.value().is_none())
                    .map(|variable| variable.index())
                    .next()
            }
        }
    }

    fn backjump(&mut self, to: LevelIndex) {
        log::trace!(target: crate::log::targets::BACKJUMP, "Backjump from {} to {}", self.levels.top().index(), to);

        for _ in 0..(self.levels.top().index() - to) {
            let the_level = self.levels.pop().expect("lost level");
            log::trace!(target: crate::log::targets::BACKJUMP, "To clear: {:?}", the_level.literals().collect::<Vec<_>>());
            for literal in the_level.literals() {
                self.variables.retract_valuation(literal.index());
            }
        }
        self.variables.clear_consequences(to);
    }
}
