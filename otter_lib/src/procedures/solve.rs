use rand::{seq::IteratorRandom, Rng};

use crate::{
    context::Context,
    db::keys::ChoiceIndex,
    dispatch::{
        self,
        comment::{self},
        delta::{self},
        report::{self},
        Dispatch,
    },
    misc::log::targets::{self},
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
    pub fn clear_choices(&mut self) {
        self.backjump(0);
    }

    pub fn solve(&mut self) -> Result<report::Solve, err::Context> {
        let this_total_time = std::time::Instant::now();

        self.preprocess()?;

        let time_limit = self.config.time_limit;

        'solve_loop: loop {
            self.counters.iterations += 1;
            log::trace!("Iteration {}", self.counters.iterations);

            self.counters.time = this_total_time.elapsed();
            if time_limit.is_some_and(|limit| self.counters.time > limit) {
                self.tx.send(Dispatch::SolveComment(comment::Solve::TimeUp));
                return Ok(self.report());
            }

            match self.expand()? {
                gen::Expansion::Proof(key, literal) => {
                    self.status = gen::Solve::Proof;

                    self.backjump(0);

                    self.note_literal(literal, gen::src::Literal::Resolution(key));
                    self.q_literal(literal)?;
                    continue 'solve_loop;
                }
                gen::Expansion::AssertingClause(key, literal) => {
                    self.status = gen::Solve::AssertingClause;

                    let the_clause = self.clause_db.get(key)?;
                    let index = self.backjump_level(the_clause.literals())?;
                    self.backjump(index);

                    self.note_literal(literal, gen::src::Literal::Forced(key));
                    self.q_literal(literal)?;

                    self.conflict_ceremony()?;
                    continue 'solve_loop;
                }

                gen::Expansion::Conflict => break 'solve_loop,

                gen::Expansion::Exhausted => {
                    //
                    match self.make_choice()? {
                        gen::Choice::Made => continue 'solve_loop,
                        gen::Choice::Exhausted => break 'solve_loop,
                    }
                }
            }
        }
        self.tx.send(Dispatch::Finish);
        Ok(self.report())
    }

    /// Expand queued consequences:
    /// Performs an analysis on apparent conflict.
    pub fn expand(&mut self) -> Result<gen::Expansion, err::Context> {
        'expansion: while let Some((literal, _)) = self.get_consequence() {
            match unsafe { self.bcp(literal) } {
                Ok(()) => {}
                Err(err::BCP::CorruptWatch) => return Err(err::Context::BCP),
                Err(err::BCP::Conflict(key)) => {
                    //
                    if !self.literal_db.choice_made() {
                        self.status = gen::Solve::NoSolution;

                        let delta = delta::Variable::Unsatisfiable(key);
                        self.tx.send(Dispatch::VariableDB(delta));

                        return Ok(gen::Expansion::Conflict);
                    }

                    let analysis_result = self.conflict_analysis(key)?;

                    match analysis_result {
                        gen::Analysis::FundamentalConflict => {
                            panic!("impossible");
                            // Analysis is only called when some decision has been made, for now
                        }

                        gen::Analysis::MissedImplication(key, literal) => {
                            let the_clause = self.clause_db.get(key)?;

                            let index = self.backjump_level(the_clause.literals())?;
                            self.backjump(index);

                            self.q_literal(literal)?;
                            self.note_literal(literal, gen::src::Literal::Missed(key));

                            continue 'expansion;
                        }

                        gen::Analysis::Proof(key, literal) => {
                            return Ok(gen::Expansion::Proof(key, literal));
                        }

                        gen::Analysis::AssertingClause(key, literal) => {
                            return Ok(gen::Expansion::AssertingClause(key, literal));
                        }
                    }
                }
            }
        }
        Ok(gen::Expansion::Exhausted)
    }

    // The config is (re)borrowed to shorten conditions
    pub fn conflict_ceremony(&mut self) -> Result<(), err::Context> {
        let config = &self.config;
        self.counters.conflicts += 1;
        self.counters.conflicts_in_memory += 1;

        if self.counters.conflicts_in_memory % (config.luby_u * self.counters.luby.current()) == 0 {
            self.counters.luby.next();

            self.tx.send(Dispatch::Stats(dispatch::stat::Count::ICD(
                self.counters.iterations,
                self.counters.conflicts,
                self.counters.choices,
            )));
            self.tx.send(Dispatch::Stats(dispatch::stat::Count::Time(
                self.counters.time,
            )));

            if config.restarts_ok {
                self.backjump(0);
                self.counters.restarts += 1;
                self.counters.conflicts_in_memory = 0;
            }

            let config = &self.config;
            if config.reductions_ok && ((self.counters.restarts % config.reduction_interval) == 0) {
                log::debug!(target: targets::REDUCTION, "Reduction after {} restarts", self.counters.restarts);
                self.clause_db.reduce()?;
            }
        }
        Ok(())
    }

    pub fn make_choice(&mut self) -> Result<gen::Choice, err::Queue> {
        match self.get_unassigned() {
            Some(choice_id) => {
                self.counters.choices += 1;

                let choice_literal = {
                    let previous_value = self.variable_db.previous_value_of(choice_id);
                    Literal::new(choice_id, previous_value)
                };
                log::trace!("Choice {choice_literal}");
                self.literal_db.note_choice(choice_literal);
                self.q_literal(choice_literal)?;

                self.status = gen::Solve::ChoiceMade;
                Ok(gen::Choice::Made)
            }
            None => {
                self.status = gen::Solve::FullValuation;
                Ok(gen::Choice::Exhausted)
            }
        }
    }

    pub fn get_unassigned(&mut self) -> Option<ChoiceIndex> {
        match self
            .counters
            .rng
            .gen_bool(self.config.random_choice_frequency)
        {
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

    pub fn backjump(&mut self, to: ChoiceIndex) {
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

    /// The second highest choice index from the given literals, or 0
    /// Aka. The backjump level for a slice of an asserting slice of literals/clause
    // Work through the clause, keeping an ordered record of the top two decision levels: (second_to_top, top)
    pub fn backjump_level(&self, literals: &[Literal]) -> Result<ChoiceIndex, err::Context> {
        let mut top_two = (None, None);
        for literal in literals {
            let Some(dl) = self.variable_db.choice_index_of(literal.var()) else {
                log::error!(target: targets::BACKJUMP, "{literal} was not chosen");
                return Err(err::Context::Backjump);
            };

            match top_two {
                (_, None) => top_two.1 = Some(dl),
                (_, Some(the_top)) if dl > the_top => {
                    top_two.0 = top_two.1;
                    top_two.1 = Some(dl);
                }
                (None, _) => top_two.0 = Some(dl),
                (Some(second_to_top), _) if dl > second_to_top => top_two.0 = Some(dl),
                _ => {}
            }
        }

        match top_two {
            (None, _) => Ok(0),
            (Some(second_to_top), _) => Ok(second_to_top),
        }
    }
}