use rand::{seq::IteratorRandom, Rng};

use crate::{
    config::Config,
    context::{
        analysis::AnalysisResult,
        core::{ContextFailure, StepInfo},
        level::LevelIndex,
        Context, Report, SolveStatus,
    },
    structures::{
        clause::Clause,
        literal::{Literal, LiteralSource},
        variable::{list::VariableList, VariableId},
    },
};

impl Context {
    pub fn solve(&mut self) -> Result<Report, ContextFailure> {
        let this_total_time = std::time::Instant::now();

        self.preprocess();

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
                Ok(_) => continue 'solve_loop,
                Err(_) => {
                    break 'solve_loop Ok(self.report());
                }
            }
        }
    }

    pub fn step(&mut self, config: &Config) -> Result<(), StepInfo> {
        self.counters.iterations += 1;

        'search: while let Some((literal, _source, _)) = self.variables.get_consequence() {
            match self.BCP(literal) {
                Ok(()) => {}
                Err(key) => {
                    let Ok(analysis_result) = self.conflict_analysis(key, config) else {
                        log::error!(target: crate::log::targets::STEP, "Conflict analysis failed.");
                        panic!("Analysis failed")
                    };

                    match analysis_result {
                        AnalysisResult::FundamentalConflict(key) => {
                            self.status = SolveStatus::NoSolution(key);

                            return Err(StepInfo::Conflict(key));
                        }

                        AnalysisResult::QueueConflict(key) => {
                            self.status = SolveStatus::NoSolution(key);

                            return Err(StepInfo::Conflict(key));
                        }

                        AnalysisResult::Proof(key, literal) => {
                            self.status = SolveStatus::Proof(key);

                            self.backjump(0);

                            match self.q_literal(literal, LiteralSource::Resolution(key)) {
                                Ok(()) => {}
                                Err(key) => return Err(StepInfo::QueueProof(key)),
                            }
                        }

                        AnalysisResult::MissedImplication(key, literal) => {
                            self.status = SolveStatus::MissedImplication(key);

                            let the_clause = self.clause_store.get(key);

                            let missed_level = self.backjump_level(the_clause.literal_slice());

                            self.backjump(missed_level);

                            match self.q_literal(literal, LiteralSource::Missed(key)) {
                                Ok(()) => {}
                                Err(key) => return Err(StepInfo::QueueConflict(key)),
                            };

                            continue 'search;
                        }

                        AnalysisResult::AssertingClause(key, literal) => {
                            self.status = SolveStatus::AssertingClause(key);

                            let the_clause = self.clause_store.get(key);

                            let backjump_index = self.backjump_level(the_clause.literal_slice());

                            self.backjump(backjump_index);

                            match self.q_literal(literal, LiteralSource::Analysis(key)) {
                                Ok(()) => {}
                                Err(key) => return Err(StepInfo::QueueConflict(key)),
                            }

                            self.conflict_ceremony(config);
                            return Ok(());
                        }
                    }
                }
            }
        }

        self.make_choice(config)
    }

    fn conflict_ceremony(&mut self, config: &Config) {
        self.counters.conflicts += 1;
        self.counters.conflicts_since_last_forget += 1;
        self.counters.conflicts_since_last_reset += 1;

        if self.it_is_time_to_restart(config.luby_constant) {
            if let Some(window) = &self.window {
                window.update_counters(&self.counters);
                window.flush();
            }

            if config.restarts_allowed {
                self.backjump(0);
                self.counters.restarts += 1;
                self.counters.conflicts_since_last_forget = 0;
            }

            if config.reduction_allowed
                && ((self.counters.restarts % config.reduction_interval) == 0)
            {
                log::debug!(target: crate::log::targets::REDUCTION, "Reduction after {} restarts", self.counters.restarts);
                self.clause_store.reduce();
            }
        }
    }

    pub fn make_choice(&mut self, config: &Config) -> Result<(), StepInfo> {
        match self.get_unassigned(config) {
            Some(choice_index) => {
                self.counters.decisions += 1;
                self.levels.get_fresh();

                log::trace!(target: crate::log::targets::STEP,
                    "Choice of {choice_index} at level {} with activity {}",
                    self.levels.top().index(),
                    self.variables.activity_of(choice_index)
                );
                let choice_literal = {
                    let choice_id = choice_index as VariableId;
                    match self.variables.get_unsafe(choice_index).previous_value() {
                        Some(polarity) => Literal::new(choice_id, polarity),
                        None => Literal::new(choice_id, self.rng.gen_bool(config.polarity_lean)),
                    }
                };
                match self.q_literal(choice_literal, LiteralSource::Choice) {
                    Ok(()) => {}
                    Err(_) => panic!("could not set choice"),
                };

                self.status = SolveStatus::ChoiceMade;
                Ok(())
            }
            None => {
                self.status = SolveStatus::FullValuation;
                Err(StepInfo::ChoicesExhausted)
            }
        }
    }

    pub fn get_unassigned(&mut self, config: &Config) -> Option<usize> {
        match self.rng.gen_bool(config.random_choice_frequency) {
            true => self
                .variables
                .iter()
                .filter(|variable| variable.value().is_none())
                .choose(&mut self.rng)
                .map(|variable| variable.index()),
            false => {
                while let Some(index) = self.variables.heap_pop_most_active() {
                    let the_variable = self.variables.get_unsafe(index);
                    if self.variables.value_of(the_variable.index()).is_none() {
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

    pub fn backjump(&mut self, to: LevelIndex) {
        log::trace!(target: crate::log::targets::BACKJUMP, "Backjump from {} to {}", self.levels.top().index(), to);

        for _ in 0..(self.levels.top().index() - to) {
            let the_level = self.levels.pop().expect("lost level");
            log::trace!(target: crate::log::targets::BACKJUMP, "To clear: {:?}", the_level.literals().collect::<Vec<_>>());
            for literal in the_level.literals() {
                self.variables.retract_valuation(literal.index());
                self.variables.heap_push(literal.index());
            }
        }
        self.variables.clear_consequences(to);
    }
}
