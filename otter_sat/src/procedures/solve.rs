//! Determines the satisfiability of the formula in a context.
//!
//! # Overview
//!
//! [solve](crate::procedures::solve) casts the conflict-driven clause-learning algorithm through a valuation relative consequence operator over formulas.
//!
//! On this operator a formula entails either itself, or a tautological consequence of itself with some additional clause.
//! And, if the operator cannot be applied, the formula is unsatisfiable.[^op-note]
//! [^op-note]: Alternatively, the operator may return some designated formula such as falsum.
//!
//! - If the formula entails itself, then inspection of the valuation is required:
//!   + If the valuation is partial (and not complete) the valuation *may be* satisfiable, though the rules of inference associated with the operator do not support the derivation of a complete valuation (and so some decision must be made).
//!   + If the valuation is complete, the formula is satisfiable (on the given valuation).
//! - If the formula entails some formula with an additional clause, then:
//!   + The formula is unsatisfiable on the given valuation, but *may be* satisfiable on some other valuation.\
//!     Specifically, there is some sub-valuation of the current valuation on which the added clause asserts some literal, and a '[backjump](crate::procedures::backjump)' may be made to that valuation.
//!
//! [solve](crate::procedures::solve), then, manages the detailed operator, whose implementation is given in [apply_consequences].
//! This amounts to applying an instance of the operator which:
//!
//! - Returns *unsatisfiable*, if it is not possible to apply the consequence relation.
//! - Returns *satisfiable*, if the formula entails itself and the valuation is complete.
//! - Makes a decision, if the formula entails itself and the valuation is partial.
//! - Backjumps to a different valuation, if the formula entails some formula with an additional clause.
//!
//! Though, at points this process may be interrupted for some other action.
//! In particular, [solve](crate::procedures::solve) may revise the valuation to some other valuation (e.g. by forgetting any decisions made) regardless of whether the formula entails some formula with an additional clause.
//!
//! Roughly, the loop is as diagrammed:
//!
//! ```none
//!           +---------------+
//!   +-------| make_decision |
//!   |       +---------------+
//!   |               ⌃
//!   |               |
//!   |               | if there is no update to the formula, and the valuation is partial
//!   |               |
//!   |               |              +-----> satisfiable, if the valuation is full
//!   ⌄   +--------------------+     |
//! --+-->| apply_consequences |-----+
//!   ⌃   +--------------------+     |
//!   |               |              +-----> unsatisfiable, if apply_consequences fails
//!   |               |
//!   |               | if a clause is added to the formula
//!   |               |
//!   |               ⌄
//!   |          +----------+
//!   +----------| backjump |
//!              +----------+
//! ```
//!
//! And, abstracting from various other bookkeeping tasks and optional actions after a context, solve is:
//!
//! ```rust,ignore
//! loop {
//!
//!     match self.apply_consequences()? {
//!         apply_consequences::ApplyConsequencesOk::FundamentalConflict => break,
//!
//!         apply_consequences::ApplyConsequencesOk::Exhausted => {
//!             //
//!             match self.make_decision()? {
//!                 decision::Ok::Made => continue,
//!                 decision::Ok::Exhausted => break,
//!             }
//!         }
//!
//!         apply_consequences::ApplyConsequencesOk::UnitClause(literal) => {
//!             self.backjump(0);
//!             self.q_literal(literal)?;
//!         }
//!
//!         apply_consequences::Ok::AssertingClause(key, literal) => {
//!             let the_clause = self.clause_db.get(&key)?;
//!             self.backjump(self.non_chronological_backjump_level(the_clause)?);
//!             self.q_literal(literal)?;
//!         }
//!     }
//!     // Additional actions after a conflict, before the next loop.
//!     ...
//! }
//! ```
//!
//! The distinction between a unit clause and clause being returned from [apply_consequence](crate::procedures::apply_consequences) is made only to avoid the overhead of accessing a clause and determing the relevant backjump level in the case of a unit clause.
//!
//! # Example
//!
//! ```rust
//! # use otter_sat::config::Config;
//! # use otter_sat::context::Context;
//! # use otter_sat::dispatch::library::report::{self};
//! # use otter_sat::structures::literal::{CLiteral, Literal};
//! let config = Config::default();
//! let mut the_context: Context = Context::from_config(config, None);
//!
//! let p = the_context.fresh_atom().unwrap();
//! let q = the_context.fresh_atom().unwrap();
//!
//! let not_p_or_q = vec![CLiteral::new(p, false), CLiteral::new(q, true)];
//! let p_or_not_q = vec![CLiteral::new(p, true), CLiteral::new(q, false)];
//! assert!(the_context.add_clause(not_p_or_q).is_ok());
//! assert!(the_context.add_clause(p_or_not_q).is_ok());
//!
//! assert!(the_context.solve().is_ok());
//! let status = the_context.report();
//!
//! assert_eq!(the_context.atom_db.value_of(p), Some(false));
//! assert_eq!(the_context.atom_db.value_of(q), Some(false));
//!
//! let p_clause = vec![CLiteral::new(p, true)];
//! assert!(the_context.add_clause(p_clause).is_err());
//!
//! the_context.clear_decisions();
//!
//! let p_clause = vec![CLiteral::new(p, true)];
//! assert!(the_context.add_clause(p_clause).is_ok());
//!
//! assert_eq!(the_context.atom_db.value_of(p), Some(true));
//!
//! assert!(the_context.solve().is_ok());
//!
//! assert_eq!(the_context.report(), report::SolveReport::Satisfiable);
//! ```
//!
//! # Literature
//!
//! The core solve procedure was developed by reading [Decision Procedures](https://doi.org/10.1007/978-3-662-50497-0)[^a]
//! and the [Handbook of satisfiability](https://www.iospress.com/catalog/books/handbook-of-satisfiability-2).[^b]
//! Though, the presentation given is original.
//!
//! [^a]: Specifically, Chapter 2 on decision procedures for propositional logic.
//! [^b]: Specifcally, chapters 3 and 4 on complete algorithms and CDCL techniques.

use crate::{
    context::{ContextState, GenericContext},
    db::consequence_q,
    dispatch::{
        library::{
            delta::{self, Delta},
            report::{self, Report, SolveReport},
            stat::Stat,
        },
        macros::{self},
        Dispatch,
    },
    procedures::{
        apply_consequences::{self},
        decision::{self},
    },
    structures::{
        clause::Clause,
        consequence::{self, Consequence},
        literal::Literal,
    },
    types::err::{self, ErrorKind},
};

impl<R: rand::Rng + std::default::Default> GenericContext<R> {
    pub fn solve(&mut self) -> Result<report::SolveReport, err::ErrorKind> {
        use crate::db::consequence_q::QPosition::{self};

        match self.state {
            ContextState::Solving => {}
            ContextState::Satisfiable | ContextState::Unsatisfiable(_) => {
                return Ok(self.report());
            }
            ContextState::Configuration | ContextState::Input => {
                for (key, clause) in self.clause_db.all_unit_clauses() {
                    if clause.unsatisfiable_on(self.atom_db.valuation()) {
                        self.state = ContextState::Unsatisfiable(key);
                        return Ok(self.report());
                    }
                }

                for (key, clause) in self.clause_db.all_nonunit_clauses() {
                    if clause.unsatisfiable_on(self.atom_db.valuation()) {
                        self.state = ContextState::Unsatisfiable(key);
                        return Ok(self.report());
                    }
                }

                match self.assert_assumptions() {
                    Ok(_) => {}
                    Err(e) => {
                        log::info!("Failed to assert assumption: {e:?}");
                        return Ok(SolveReport::Unsatisfiable);
                    }
                };

                self.state = ContextState::Solving;

                self.preprocess()?;
            }
        }

        let timer = std::time::Instant::now();

        'solve_loop: loop {
            self.counters.total_iterations += 1;
            log::trace!("Iteration {}", self.counters.total_iterations);

            self.counters.time = timer.elapsed();
            let time_limit = self.config.time_limit;
            if time_limit.is_some_and(|limit| self.counters.time > limit) {
                return Ok(report::SolveReport::TimeUp);
            }

            if let Some(callbacks) = &self.ipasir_callbacks {
                if unsafe { callbacks.call_ipasir_terminate_callback() } != 0 {
                    break 'solve_loop;
                }
            }

            match self.apply_consequences()? {
                // Non-conflict variants. These variants break or continue the solve loop.
                apply_consequences::ApplyConsequencesOk::FundamentalConflict => break 'solve_loop,

                apply_consequences::ApplyConsequencesOk::Exhausted => {
                    //
                    match self.make_decision()? {
                        decision::DecisionOk::Literal(decision) => {
                            self.literal_db.decision_made(decision);
                            let level = self.literal_db.decision_level();
                            self.value_and_queue(decision, QPosition::Back, level)?;
                            continue 'solve_loop;
                        }
                        decision::DecisionOk::Exhausted => break 'solve_loop,
                    }
                }

                // Conflict variants. These continue to the remaining contents of a loop.
                apply_consequences::ApplyConsequencesOk::UnitClause { key } => {
                    self.value_and_queue(key, QPosition::Front, self.literal_db.lower_limit())?;
                }

                apply_consequences::ApplyConsequencesOk::AssertingClause { key, literal } => {
                    self.clause_db.note_use(key);
                    macros::dispatch_bcp_delta!(self, Instance, literal, key);

                    let consequence = Consequence::from(literal, consequence::Source::BCP(key));
                    let level = self.literal_db.decision_level();
                    self.value_and_queue(literal, QPosition::Front, level)?;
                    self.record_consequence(consequence);
                }
            }

            self.counters.total_conflicts += 1;
            self.counters.fresh_conflicts += 1;

            if self.luby_fresh_conflict_interrupt() {
                self.counters.luby.next();

                macros::dispatch_stats!(self);

                if self.config.switch.restart {
                    self.backjump(self.literal_db.lower_limit());
                    self.clause_db.refresh_heap();
                    self.counters.fresh_conflicts = 0;
                    self.counters.restarts += 1;
                };

                if self.restart_interrupt() {
                    self.clause_db.reduce_by(
                        self.clause_db.current_addition_count() / 2,
                        &self.ipasir_callbacks,
                    );
                }
            } else if self.conflict_total_interrupt() {
                self.clause_db.reduce_by(
                    self.clause_db.current_addition_count() / 2,
                    &self.ipasir_callbacks,
                )?;
            }
        }

        macros::dispatch_finish!(self);
        Ok(self.report())
    }

    fn assert_assumptions(&mut self) -> Result<(), ErrorKind> {
        if self.literal_db.decision_is_made() {
            panic!("! Asserting assumptions while a decision has been made.");
            return Ok(());
        }

        match self.config.literal_db.stacked_assumptions {
            true => {
                let assumption_count = self.literal_db.lower_limit();

                for index in 0..assumption_count {
                    let assumption = unsafe { self.literal_db.decision_unchecked(index) };
                    match self.atom_db.value_of(assumption.atom()) {
                        None => match self.value_and_queue(
                            assumption,
                            consequence_q::QPosition::Back,
                            index,
                        ) {
                            Ok(consequence_q::ConsequenceQueueOk::Qd) => continue,
                            _ => {
                                return Err(err::ErrorKind::from(
                                    err::ClauseDBError::ValuationConflict,
                                ))
                            }
                        },

                        Some(v) if v == assumption.polarity() => {
                            continue;
                        }

                        Some(_) => {
                            return Err(err::ErrorKind::from(err::ClauseDBError::ValuationConflict))
                        }
                    }
                }
            }

            false => {
                let assumption_count = self.literal_db.flat_assumptions().len();

                for index in 0..assumption_count {
                    let assumption = self.literal_db.flat_assumptions()[index];
                    match self.atom_db.value_of(assumption.atom()) {
                        None => match self.value_and_queue(
                            assumption,
                            consequence_q::QPosition::Back,
                            self.literal_db.lower_limit(),
                        ) {
                            Ok(consequence_q::ConsequenceQueueOk::Qd) => continue,
                            _ => {
                                return Err(err::ErrorKind::from(
                                    err::ClauseDBError::ValuationConflict,
                                ))
                            }
                        },

                        Some(v) if v == assumption.polarity() => {
                            // Must be at zero for an assumption, so there's nothing to do
                            continue;
                        }

                        Some(_) => {
                            return Err(err::ErrorKind::from(err::ClauseDBError::ValuationConflict))
                        }
                    }
                }
            }
        }

        Ok(())
    }
}
