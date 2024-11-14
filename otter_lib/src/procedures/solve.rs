use rand::{seq::IteratorRandom, Rng};

use crate::{
    config::Config,
    context::Context,
    db::keys::ChoiceIndex,
    dispatch::{
        self,
        comment::{self},
        delta::{self},
        report::{self},
        Dispatch,
    },
    procedures::{analysis::AnalysisResult, bcp::BCPErr},
    structures::{
        clause::Clause,
        literal::{Literal, LiteralT},
        variable::Variable,
    },
    types::{
        err::{self},
        gen::{self},
    },
};

impl Context {
    pub fn solve(&mut self) -> Result<report::Solve, err::Context> {
        let this_total_time = std::time::Instant::now();

        match self.preprocess() {
            Ok(()) => {}
            Err(_) => panic!("Preprocessing failure"),
        };

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
                Ok(gen::Step::One) => continue 'solve_loop,
                Ok(gen::Step::ChoiceMade) => continue 'solve_loop,
                Ok(gen::Step::ChoicesExhausted) => break 'solve_loop Ok(self.report()),
                Ok(gen::Step::Conflict) => break 'solve_loop Ok(self.report()),

                Err(err::Step::Backfall) => panic!("Backjumping failed"),
                Err(err::Step::AnalysisFailure) => panic!("Analysis failed"),
                Err(err::Step::ChoiceFailure) => panic!("Choice failure"),
                Err(e) => panic!("{e:?}"),
            }
        }
    }

    pub fn step(&mut self, config: &Config) -> Result<gen::Step, err::Step> {
        self.counters.iterations += 1;
        log::trace!("Step {}", self.counters.iterations);

        'search: while let Some((literal, _)) = self.get_consequence() {
            match unsafe { self.bcp(literal) } {
                Ok(()) => {}
                Err(BCPErr::Conflict(key)) => {
                    //
                    if !self.literal_db.choice_made() {
                        self.status = gen::SolveStatus::NoSolution;

                        let report = delta::Variable::Falsum(literal);
                        self.tx.send(Dispatch::VariableDB(report));

                        return Ok(gen::Step::Conflict);
                    }

                    let Ok(analysis_result) = self.conflict_analysis(key, config) else {
                        log::error!(target: crate::log::targets::STEP, "Conflict analysis failed.");
                        return Err(err::Step::AnalysisFailure);
                    };

                    match analysis_result {
                        AnalysisResult::FundamentalConflict => {
                            panic!("Impossible");
                            // Analysis is only called when some decision has been made, for now
                            // return Ok(gen::Step::Conflict);
                        }

                        AnalysisResult::Proof(key, literal) => {
                            self.status = gen::SolveStatus::Proof;

                            self.backjump(0);

                            let Ok(gen::QStatus::Qd) = self.q_literal(literal) else {
                                return Err(err::Step::QueueProof(key));
                            };
                        }

                        AnalysisResult::MissedImplication(key, literal) => {
                            self.status = gen::SolveStatus::MissedImplication;

                            let Ok(the_clause) = self.clause_db.get(key) else {
                                panic!("mi");
                            };

                            match self.backjump_level(the_clause.literals()) {
                                None => return Err(err::Step::Backfall),
                                Some(index) => self.backjump(index),
                            }

                            let Ok(gen::QStatus::Qd) = self.q_literal(literal) else {
                                return Err(err::Step::QueueConflict(key));
                            };
                            self.note_literal(literal, gen::LiteralSource::Missed(key));

                            continue 'search;
                        }

                        AnalysisResult::AssertingClause(key, literal) => {
                            self.status = gen::SolveStatus::AssertingClause;

                            let Ok(the_clause) = self.clause_db.get(key) else {
                                println!("{key:?}");
                                panic!("here, asserting")
                            };

                            match self.backjump_level(the_clause.literals()) {
                                None => return Err(err::Step::Backfall),
                                Some(index) => self.backjump(index),
                            }

                            match self.q_literal(literal) {
                                Ok(gen::QStatus::Qd) => {
                                    self.note_literal(literal, gen::LiteralSource::Analysis(key));
                                }
                                Err(_) => return Err(err::Step::QueueConflict(key)),
                            }

                            self.conflict_ceremony(config)?;
                            return Ok(gen::Step::One);
                        }
                    }
                }
                Err(BCPErr::CorruptWatch) => return Err(err::Step::BCPFailure),
            }
        }

        self.make_choice(config)
    }
}

impl Context {
    pub fn clear_choices(&mut self) {
        self.backjump(0);
    }
}

impl Context {
    fn conflict_ceremony(&mut self, config: &Config) -> Result<(), err::Step> {
        self.counters.conflicts += 1;
        self.counters.conflicts_in_memory += 1;

        if self.counters.conflicts_in_memory
            % (config.luby_constant * self.counters.luby.current()) as usize
            == 0
        {
            self.counters.luby.next();
            {
                use dispatch::stat::Count;
                self.tx.send(Dispatch::Stats(Count::ICD(
                    self.counters.iterations,
                    self.counters.conflicts,
                    self.counters.choices,
                )));
                self.tx
                    .send(Dispatch::Stats(Count::Time(self.counters.time)));
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
                self.clause_db.reduce()?;
            }
        }
        Ok(())
    }

    fn make_choice(&mut self, config: &Config) -> Result<gen::Step, err::Step> {
        match self.get_unassigned(config) {
            Some(choice_id) => {
                self.counters.choices += 1;

                let choice_literal = {
                    let previous_value = self.variable_db.previous_value_of(choice_id);
                    Literal::new(choice_id, previous_value)
                };
                log::trace!("Choice {choice_literal}");
                self.literal_db.make_choice(choice_literal);
                let Ok(gen::QStatus::Qd) = self.q_literal(choice_literal) else {
                    return Err(err::Step::ChoiceFailure);
                };

                self.status = gen::SolveStatus::ChoiceMade;
                Ok(gen::Step::ChoiceMade)
            }
            None => {
                self.status = gen::SolveStatus::FullValuation;
                Ok(gen::Step::ChoicesExhausted)
            }
        }
    }

    fn get_unassigned(&mut self, config: &Config) -> Option<ChoiceIndex> {
        match self.counters.rng.gen_bool(config.random_choice_frequency) {
            true => self
                .variable_db
                .valuation()
                .iter()
                .enumerate()
                .filter_map(|(i, v)| match v {
                    None => Some(i as ChoiceIndex),
                    _ => None,
                })
                .choose(&mut self.counters.rng),
            false => {
                while let Some(index) = self.variable_db.heap_pop_most_active() {
                    // let the_variable = self.variable_db.get_unsafe(index);
                    if self.variable_db.value_of(index as Variable).is_none() {
                        return Some(index);
                    }
                }
                self.variable_db
                    .valuation()
                    .iter()
                    .enumerate()
                    .filter_map(|(i, v)| match v {
                        None => Some(i as Variable),
                        _ => None,
                    })
                    .next()
            }
        }
    }

    fn backjump(&mut self, to: ChoiceIndex) {
        // log::trace!(target: crate::log::targets::BACKJUMP, "Backjump from {} to {}", self.levels.index(), to);

        for _ in 0..(self.literal_db.choice_count() - to) {
            self.variable_db
                .drop_value(self.literal_db.last_choice().var());
            for (_, literal) in self.literal_db.last_consequences() {
                self.variable_db.drop_value(literal.var());
            }
            self.literal_db.forget_last_choice();
        }
        self.clear_consequences(to);
    }
}
