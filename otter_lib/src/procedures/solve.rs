use rand::{seq::IteratorRandom, Rng};

use crate::{
    context::Context,
    db::keys::ChoiceIndex,
    dispatch::{
        library::{
            delta::{self, Delta},
            report::{self, Report},
            stat::Stat,
        },
        Dispatch,
    },
    misc::log::targets::{self},
    structures::{
        clause::ClauseT,
        literal::{Literal, LiteralT},
        valuation::Valuation,
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

        'solve_loop: loop {
            self.counters.iterations += 1;
            log::trace!("Iteration {}", self.counters.iterations);

            self.counters.time = this_total_time.elapsed();
            let time_limit = self.config.time_limit;
            if time_limit.is_some_and(|limit| self.counters.time > limit) {
                return Ok(report::Solve::TimeUp);
            }

            match self.expand()? {
                gen::Expansion::Proof(literal) => {
                    self.status = gen::Solve::Proof;

                    self.backjump(0);

                    self.literal_db
                        .record_literal(literal, gen::src::Literal::Resolution);
                    self.q_literal(literal)?;
                    continue 'solve_loop;
                }
                gen::Expansion::AssertingClause(key, literal) => {
                    self.status = gen::Solve::AssertingClause;

                    let the_clause = self.clause_db.get(key)?;
                    let index = self.backjump_level(the_clause)?;
                    self.backjump(index);

                    self.clause_db.note_use(key);
                    if let Some(dispatcher) = &self.dispatcher {
                        let delta = delta::BCP::Instance {
                            via: key,
                            to: literal,
                        };
                        dispatcher(Dispatch::Delta(Delta::BCP(delta)));
                    }
                    self.literal_db
                        .record_literal(literal, gen::src::Literal::BCP(key));
                    self.q_literal(literal)?;

                    self.counters.conflicts += 1;
                    self.counters.fresh_conflicts += 1;

                    if self.scheduled_luby_interrupt() {
                        self.counters.luby.next();
                        self.conflict_dispatch();

                        if self.config.switch.restart {
                            self.restart()
                        };

                        if self.scheduled_by_luby() {
                            self.clause_db.reduce();
                        }
                    } else if self.scheduled_by_conflicts() {
                        self.clause_db.reduce()?;
                    }

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
        if let Some(dispatcher) = &self.dispatcher {
            dispatcher(Dispatch::Report(Report::Finish));
        }
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

                        if let Some(dispatcher) = &self.dispatcher {
                            let delta = delta::VariableDB::Unsatisfiable(key);
                            dispatcher(Dispatch::Delta(Delta::VariableDB(delta)));
                        }

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

                            let index = self.backjump_level(the_clause)?;
                            self.backjump(index);

                            self.q_literal(literal)?;

                            if let Some(dispatcher) = &self.dispatcher {
                                let delta = delta::BCP::Instance {
                                    via: key,
                                    to: literal,
                                };
                                dispatcher(Dispatch::Delta(Delta::BCP(delta)));
                            }
                            self.literal_db
                                .record_literal(literal, gen::src::Literal::BCP(key));

                            continue 'expansion;
                        }

                        gen::Analysis::UnitClause(literal) => {
                            return Ok(gen::Expansion::Proof(literal));
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

    pub fn conflict_dispatch(&self) {
        if let Some(dispatcher) = &self.dispatcher {
            dispatcher(Dispatch::Stat(Stat::Iterations(self.counters.iterations)));
            dispatcher(Dispatch::Stat(Stat::Chosen(self.counters.choices)));
            dispatcher(Dispatch::Stat(Stat::Conflicts(self.counters.conflicts)));
            dispatcher(Dispatch::Stat(Stat::Time(self.counters.time)));
        }
    }

    pub fn restart(&mut self) {
        self.backjump(0);
        self.clause_db.reset_heap();
        self.counters.restarts += 1;
        self.counters.fresh_conflicts = 0;
    }

    #[inline(always)]
    pub fn scheduled_luby_interrupt(&self) -> bool {
        self.counters.fresh_conflicts % (self.config.luby_u * self.counters.luby.current()) == 0
    }

    #[inline(always)]
    pub fn scheduled_by_conflicts(&self) -> bool {
        self.config
            .scheduler
            .conflict
            .is_some_and(|interval| (self.counters.conflicts % (interval as usize)) == 0)
    }

    pub fn scheduled_by_luby(&self) -> bool {
        self.config
            .scheduler
            .luby
            .is_some_and(|interval| (self.counters.restarts % (interval as usize)) == 0)
    }

    pub fn make_choice(&mut self) -> Result<gen::Choice, err::Queue> {
        match self.get_unassigned() {
            Some(choice_id) => {
                self.counters.choices += 1;

                let choice_literal = {
                    if self.config.switch.phase_saving {
                        let previous_value = self.variable_db.previous_value_of(choice_id);
                        Literal::new(choice_id, previous_value)
                    } else {
                        Literal::new(
                            choice_id,
                            self.counters.rng.gen_bool(self.config.polarity_lean),
                        )
                    }
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

    pub fn get_unassigned(&mut self) -> Option<Variable> {
        match self
            .counters
            .rng
            .gen_bool(self.config.random_choice_frequency)
        {
            true => self
                .variable_db
                .valuation()
                .unvalued_variables()
                .choose(&mut self.counters.rng),
            false => {
                while let Some(index) = self.variable_db.heap_pop_most_active() {
                    // let the_variable = self.variable_db.get_unsafe(index);
                    if self.variable_db.value_of(index as Variable).is_none() {
                        return Some(index);
                    }
                }
                self.variable_db.valuation().unvalued_variables().next()
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
    pub fn backjump_level<'l>(&self, clause: &impl ClauseT) -> Result<ChoiceIndex, err::Context> {
        match clause.size() {
            0 => panic!("impossible"),
            1 => Ok(0),
            _ => {
                let mut top_two = (None, None);
                for literal in clause.literals() {
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
    }
}
