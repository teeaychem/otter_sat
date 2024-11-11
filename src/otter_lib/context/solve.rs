use std::ops::Deref;

use log::log;
use rand::{seq::IteratorRandom, Rng};

use crate::{
    config::Config,
    context::{
        analysis::AnalysisResult,
        core::{ContextFailure, StepInfo},
        stores::LevelIndex,
        Context, SolveStatus,
    },
    dispatch::{
        self,
        comment::{self},
        delta,
        report::{self},
        Dispatch,
    },
    structures::{
        clause::Clause,
        literal::{Literal, LiteralSource, LiteralTrait},
        variable::{list::VariableList, VariableId, BCP::BCPErr},
    },
    types::errs::{self},
};

use super::stores::variable::QStatus;

impl Context {
    pub fn solve(&mut self) -> Result<report::Solve, ContextFailure> {
        let this_total_time = std::time::Instant::now();

        match self.preprocess() {
            Ok(()) => {}
            Err(_) => panic!("Preprocessing failure"),
        };

        if self.config.io.show_stats {
            if let Some(window) = &mut self.window {
                window.draw_window(&self.config);
            }
        }

        let config_clone = self.config.clone();
        let time_limit = config_clone.time_limit;

        'solve_loop: loop {
            self.counters.time = this_total_time.elapsed();
            if time_limit.is_some_and(|limit| self.counters.time > limit) {
                let comment = comment::Solve::TimeUp;
                self.tx.send(Dispatch::SolveComment(comment));
                return Ok(self.report());
            }

            match self.step(&config_clone) {
                Ok(StepInfo::One) => continue 'solve_loop,
                Ok(StepInfo::ChoiceMade) => continue 'solve_loop,
                Ok(StepInfo::ChoicesExhausted) => break 'solve_loop Ok(self.report()),
                Ok(StepInfo::Conflict) => break 'solve_loop Ok(self.report()),

                Err(errs::Step::Backfall) => panic!("Backjumping failed"),
                Err(errs::Step::AnalysisFailure) => panic!("Analysis failed"),
                Err(errs::Step::ChoiceFailure) => panic!("Choice failure"),
                Err(e) => panic!("{e:?}"),
            }
        }
    }

    pub fn step(&mut self, config: &Config) -> Result<StepInfo, errs::Step> {
        self.counters.iterations += 1;
        log::trace!("Step {}", self.counters.iterations);

        'search: while let Some((literal, _)) = self.variables.get_consequence() {
            match self.BCP(literal) {
                Ok(()) => {}
                Err(BCPErr::Conflict(key)) => {
                    //
                    if !self.levels.decision_made() {
                        self.status = SolveStatus::NoSolution;

                        let report = delta::Variable::Falsum(literal);
                        self.tx.send(Dispatch::VariableDB(report));

                        return Ok(StepInfo::Conflict);
                    }

                    let Ok(analysis_result) = self.conflict_analysis(key, config) else {
                        log::error!(target: crate::log::targets::STEP, "Conflict analysis failed.");
                        return Err(errs::Step::AnalysisFailure);
                    };

                    match analysis_result {
                        AnalysisResult::FundamentalConflict => {
                            panic!("Impossible");
                            // Analysis is only called when some decision has been made, for now
                            self.status = SolveStatus::NoSolution;
                            return Ok(StepInfo::Conflict);
                        }

                        AnalysisResult::Proof(key, literal) => {
                            self.status = SolveStatus::Proof;

                            self.backjump(0);

                            let Ok(QStatus::Qd) = self.q_literal(literal) else {
                                return Err(errs::Step::QueueProof(key));
                            };
                        }

                        AnalysisResult::MissedImplication(key, literal) => {
                            self.status = SolveStatus::MissedImplication;

                            let Ok(the_clause) = self.clause_store.get(key) else {
                                panic!("mi");
                            };

                            match self.backjump_level(the_clause.deref()) {
                                None => return Err(errs::Step::Backfall),
                                Some(index) => self.backjump(index),
                            }

                            let Ok(QStatus::Qd) = self.q_literal(literal) else {
                                return Err(errs::Step::QueueConflict(key));
                            };
                            self.note_literal(
                                literal.canonical(),
                                LiteralSource::Missed(key),
                                Vec::default(),
                            );

                            continue 'search;
                        }

                        AnalysisResult::AssertingClause(key, literal) => {
                            self.status = SolveStatus::AssertingClause;

                            let Ok(the_clause) = self.clause_store.get(key) else {
                                println!("{key:?}");
                                panic!("here, asserting")
                            };

                            match self.backjump_level(the_clause.deref()) {
                                None => return Err(errs::Step::Backfall),
                                Some(index) => self.backjump(index),
                            }

                            match self.q_literal(literal) {
                                Ok(QStatus::Qd) => {
                                    self.note_literal(
                                        literal.canonical(),
                                        LiteralSource::Analysis(key),
                                        Vec::default(),
                                    );
                                }
                                Err(_) => return Err(errs::Step::QueueConflict(key)),
                            }

                            self.conflict_ceremony(config)?;
                            return Ok(StepInfo::One);
                        }
                    }
                }
                Err(BCPErr::CorruptWatch) => return Err(errs::Step::BCPFailure),
            }
        }

        self.make_choice(config)
    }
}

impl Context {
    pub fn clear_decisions(&mut self) {
        self.backjump(0);
    }
}

impl Context {
    fn conflict_ceremony(&mut self, config: &Config) -> Result<(), errs::Step> {
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

    fn make_choice(&mut self, config: &Config) -> Result<StepInfo, errs::Step> {
        match self.get_unassigned(config) {
            Some(choice_index) => {
                self.counters.decisions += 1;

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
                log::trace!("Choice {choice_literal}");
                self.levels.make_choice(choice_literal);
                let Ok(QStatus::Qd) = self.q_literal(choice_literal) else {
                    return Err(errs::Step::ChoiceFailure);
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
        // log::trace!(target: crate::log::targets::BACKJUMP, "Backjump from {} to {}", self.levels.index(), to);

        for _ in 0..(self.levels.decision_count() - to) {
            self.variables
                .retract_valuation(self.levels.current_choice().index());
            for literal in self.levels.current_consequences().iter().map(|(_, l)| *l) {
                self.variables.retract_valuation(literal.index());
            }
            self.levels.forget_current_choice();
        }
        self.variables.clear_consequences(to);
    }
}
