//! Determines the satisfiability of the formula in a context.

use crate::{
    context::GenericContext,
    db::ClauseKey,
    dispatch::{
        library::{
            delta::{self, Delta},
            report::{self, Report},
            stat::Stat,
        },
        Dispatch,
    },
    procedures::{
        apply_consequences::{self},
        choice::{self},
    },
    structures::literal::{self},
    types::err::{self},
};

/// A macro to simplify dispatches.
macro_rules! send_stats {
    ($self:ident ) => {{
        if let Some(dispatcher) = &$self.dispatcher {
            dispatcher(Dispatch::Stat(Stat::Iterations(
                $self.counters.total_iterations,
            )));
            dispatcher(Dispatch::Stat(Stat::Chosen($self.counters.total_choices)));
            dispatcher(Dispatch::Stat(Stat::Conflicts(
                $self.counters.total_conflicts,
            )));
            dispatcher(Dispatch::Stat(Stat::Time($self.counters.time)));
        }
    }};
}

impl<R: rand::Rng + std::default::Default> GenericContext<R> {
    pub fn solve(&mut self) -> Result<report::Solve, err::Context> {
        let this_total_time = std::time::Instant::now();

        self.preprocess()?;

        'solve_loop: loop {
            self.counters.total_iterations += 1;
            log::trace!("Iteration {}", self.counters.total_iterations);

            self.counters.time = this_total_time.elapsed();
            let time_limit = self.config.time_limit;
            if time_limit.is_some_and(|limit| self.counters.time > limit) {
                return Ok(report::Solve::TimeUp);
            }

            let conflict_found;

            match self.apply_consequences()? {
                apply_consequences::Ok::FundamentalConflict => break 'solve_loop,

                apply_consequences::Ok::UnitClause(key) => {
                    self.backjump(0);
                    let ClauseKey::Unit(literal) = key else {
                        panic!("non-unit key");
                    };

                    self.q_literal(literal)?;
                    conflict_found = true;
                }

                apply_consequences::Ok::AssertingClause(key, literal) => {
                    let the_clause = unsafe { self.clause_db.get_unchecked(&key)? };
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
                self.counters.total_conflicts += 1;
                self.counters.fresh_conflicts += 1;

                if self.luby_fresh_conflict_interrupt() {
                    self.counters.luby.next();

                    send_stats!(self);

                    if self.config.switch.restart {
                        self.restart()
                    };

                    if self.restart_interrupt() {
                        self.clause_db
                            .reduce_by(self.clause_db.current_addition_count() / 2);
                    }
                } else if self.conflict_total_interrupt() {
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

    pub fn restart(&mut self) {
        self.backjump(0);
        self.clause_db.refresh_heap();
        self.counters.restarts += 1;
        self.counters.fresh_conflicts = 0;
    }
}
