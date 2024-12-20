use rand::{seq::IteratorRandom, Rng};

use crate::{
    context::Context,
    db::keys::{ChoiceIndex, ClauseKey},
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
        atom::Atom,
        clause::Clause,
        literal::{abLiteral, Literal},
        valuation::Valuation,
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

            let conflict_found;

            match self.apply_consequences()? {
                gen::Expansion::Conflict => break 'solve_loop,

                gen::Expansion::UnitClause(key) => {
                    self.backjump(0);
                    let ClauseKey::Unit(the_literal) = key else {
                        panic!("non-unit key");
                    };

                    self.q_literal(the_literal)?;
                    conflict_found = true;
                }

                gen::Expansion::AssertingClause(key, literal) => {
                    let the_clause = self.clause_db.get_db_clause(key)?;
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
                    self.record_literal(literal, gen::src::Literal::BCP(key));
                    self.q_literal(literal)?;
                    conflict_found = true;
                }

                gen::Expansion::Exhausted => {
                    //
                    match self.make_choice()? {
                        gen::Choice::Made => continue 'solve_loop,
                        gen::Choice::Exhausted => break 'solve_loop,
                    }
                }
            }

            if conflict_found {
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
            }
        }
        if let Some(dispatcher) = &self.dispatcher {
            dispatcher(Dispatch::Report(Report::Finish));
        }
        Ok(self.report())
    }

    /// Expand queued consequences:
    /// Performs an analysis on apparent conflict.
    pub fn apply_consequences(&mut self) -> Result<gen::Expansion, err::Context> {
        'expansion: while let Some((literal, _)) = self.get_consequence() {
            match unsafe { self.bcp(literal) } {
                Ok(()) => {}
                Err(err::BCP::CorruptWatch) => return Err(err::Context::BCP),
                Err(err::BCP::Conflict(key)) => {
                    //
                    if !self.literal_db.choice_made() {
                        self.status = gen::dbStatus::Inconsistent;

                        if let Some(dispatcher) = &self.dispatcher {
                            let delta = delta::AtomDB::Unsatisfiable(key);
                            dispatcher(Dispatch::Delta(Delta::AtomDB(delta)));
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
                            let the_clause = self.clause_db.get_db_clause(key)?;

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
                            self.record_literal(literal, gen::src::Literal::BCP(key));

                            continue 'expansion;
                        }

                        gen::Analysis::UnitClause(key) => {
                            return Ok(gen::Expansion::UnitClause(key));
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
                        let previous_value = self.atom_db.previous_value_of(choice_id);
                        abLiteral::fresh(choice_id, previous_value)
                    } else {
                        abLiteral::fresh(
                            choice_id,
                            self.counters.rng.gen_bool(self.config.polarity_lean),
                        )
                    }
                };
                log::trace!("Choice {choice_literal}");
                self.literal_db.note_choice(choice_literal);
                self.q_literal(choice_literal)?;

                Ok(gen::Choice::Made)
            }
            None => {
                self.status = gen::dbStatus::Consistent;
                Ok(gen::Choice::Exhausted)
            }
        }
    }

    pub fn get_unassigned(&mut self) -> Option<Atom> {
        match self
            .counters
            .rng
            .gen_bool(self.config.random_choice_frequency)
        {
            true => self
                .atom_db
                .valuation()
                .unvalued_atoms()
                .choose(&mut self.counters.rng),
            false => {
                while let Some(index) = self.atom_db.heap_pop_most_active() {
                    if self.atom_db.value_of(index as Atom).is_none() {
                        return Some(index);
                    }
                }
                self.atom_db.valuation().unvalued_atoms().next()
            }
        }
    }

    pub fn backjump(&mut self, to: ChoiceIndex) {
        // log::trace!(target: crate::log::targets::BACKJUMP, "Backjump from {} to {}", self.levels.index(), to);

        for _ in 0..(self.literal_db.choice_count() - to) {
            self.atom_db
                .drop_value(self.literal_db.last_choice().atom());
            for (_, literal) in self.literal_db.last_consequences() {
                self.atom_db.drop_value(literal.atom());
            }
            self.literal_db.forget_last_choice();
        }
        self.clear_consequences(to);
    }

    /// The second highest choice index from the given literals, or 0
    /// Aka. The backjump level for a slice of an asserting slice of literals/clause
    // Work through the clause, keeping an ordered record of the top two decision levels: (second_to_top, top)
    pub fn backjump_level(&self, clause: &impl Clause) -> Result<ChoiceIndex, err::Context> {
        match clause.size() {
            0 => panic!("impossible"),
            1 => Ok(0),
            _ => {
                let mut top_two = (None, None);
                for literal in clause.literals() {
                    let Some(dl) = self.atom_db.choice_index_of(literal.atom()) else {
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
