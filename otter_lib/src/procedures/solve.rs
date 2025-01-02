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
    procedures::{
        apply_consequences::{self},
        choice::{self},
    },
    structures::{
        clause::Clause,
        literal::{self, Literal},
    },
    types::err::{self},
};

impl Context {
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
                apply_consequences::Ok::Conflict => break 'solve_loop,

                apply_consequences::Ok::UnitClause(key) => {
                    self.backjump(0);
                    let ClauseKey::Unit(literal) = key else {
                        panic!("non-unit key");
                    };

                    self.q_literal(literal)?;
                    conflict_found = true;
                }

                apply_consequences::Ok::AssertingClause(key, literal) => {
                    let the_clause = unsafe { self.clause_db.get_db_clause_unchecked(&key)? };
                    let index = self.backjump_level(the_clause)?;
                    self.backjump(index);

                    self.clause_db.note_use(key);
                    if let Some(dispatcher) = &self.dispatcher {
                        let delta = delta::BCP::Instance {
                            clause: key,
                            literal,
                        };
                        dispatcher(Dispatch::Delta(Delta::BCP(delta)));
                    }
                    self.record_literal(literal, literal::Source::BCP(key));
                    self.q_literal(literal)?;
                    conflict_found = true;
                }

                apply_consequences::Ok::Exhausted => {
                    //
                    match self.make_choice()? {
                        choice::Ok::Made => continue 'solve_loop,
                        choice::Ok::Exhausted => break 'solve_loop,
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
                        self.clause_db
                            .reduce_by(self.clause_db.current_addition_count() / 2);
                    }
                } else if self.scheduled_by_conflicts() {
                    self.clause_db
                        .reduce_by(self.clause_db.current_addition_count() / 2)?;
                }
            }
        }
        if let Some(dispatcher) = &self.dispatcher {
            dispatcher(Dispatch::Report(Report::Finish));
        }
        Ok(self.report())
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
        self.clause_db.refresh_heap();
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

    pub fn backjump(&mut self, to: ChoiceIndex) {
        // log::trace!(target: crate::log::targets::BACKJUMP, "Backjump from {} to {}", self.levels.index(), to);

        for _ in 0..(self.literal_db.choice_count() - to) {
            unsafe {
                self.atom_db
                    .drop_value(self.literal_db.last_choice().atom())
            };
            for (_, literal) in self.literal_db.last_consequences() {
                unsafe { self.atom_db.drop_value(literal.atom()) };
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
                    let Some(dl) = (unsafe { self.atom_db.choice_index_of(literal.atom()) }) else {
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
