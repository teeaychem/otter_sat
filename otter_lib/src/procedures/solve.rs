//! Determines the satisfiability of the formula in a context.
//!
//! # Overview
//!
//! [solve](crate::procedures::solve) casts the conflict-driven clause-learning algorithm through a valuation relative consequence operator over formulas.
//!
//! On this operator a formula entails either itself, or a tautological consequence of itself with some additional clause.
//!
//! - If the formula entails itself, then inspection of the valuation is required:
//!   + If the valuation does not contain any choices, the formula is unsatisfiable (on any valuation).
//!   + If the valuation is complete, the formula is satisfiable (on the given valuation).
//!   + If the valuation is partial (and not complete) the valuation *may be* satisfiable, though the rules of inference associated with the operator do not support the derivation of a complete valuation (and so some choice must be made).
//! - If the formula entails some formula with an additional clause, then:
//!   + The formula is unsatisfiable on the given valuation, but *may be* satisfiable on some other valuation.
//!
//! [solve](crate::procedures::solve), then, manages the detailed operator, whose implementation is given in [apply_consequences].
//! This amounts to applying an instance of the operator:
//!
//! - Returning *unsatisfiable*, if the formula entails itself and the valuation does not contain any choices.
//! - Returning *satisfiable*, if the formula entails itself and the valuation is complete.
//! - Making a choice, if the formula entails itself and the valuation is partial.
//! - 'Backjumping' to a different valuation, if the formula entails some formula with an additional clause.
//!
//! Though, at points this process may be interrupted for some other action.
//! In particular, [solve](crate::procedures::solve) may revise the valuation to some other valuation (e.g. by forgetting any choices made) regardless of whether the formula entails some formula with an additional clause.
//!
//! # [solve](crate::procedures::solve)
//!
//! Abstracting from various other bookkeeping tasks and optional actions after a context, [solve](crate::procedures::solve) is as follows:
//!
//! ```rust,ignore
//! loop {
//!
//!     match self.apply_consequences()? {
//!         apply_consequences::Ok::FundamentalConflict => break,
//!
//!         apply_consequences::Ok::Exhausted => {
//!             //
//!             match self.make_choice()? {
//!                 choice::Ok::Made => continue,
//!                 choice::Ok::Exhausted => break,
//!             }
//!         }
//!
//!         apply_consequences::Ok::UnitClause(literal) => {
//!             self.backjump(0);
//!             self.q_literal(literal)?;
//!         }
//!
//!         apply_consequences::Ok::AssertingClause(key, literal) => {
//!             let the_clause = self.clause_db.get(&key)?;
//!             self.backjump(self.backjump_level(the_clause)?);
//!             self.q_literal(literal)?;
//!         }
//!     }
//!     // Additional actions after a conflict, before the next loop.
//!     ...
//! }
//! ```
//!
//! The distinction between a unit clause and clause being returned from [apply_consequence] is made only to avoid the overhead of accessing a clause and determing the relevant backjump level in the case of a unit clause.
//!
//! # Example
//!
//! ```rust
//! # fn value_of(variable: &str, context: &Context) -> Option<bool> {
//! #     let mut the_value = None;
//! #     if context.atom_db.valuation_string().contains(variable) {
//! #         the_value = Some(true)
//! #     }
//! #     if context
//! #         .atom_db
//! #         .valuation_string()
//! #         .contains(format!("-{variable}").as_str())
//! #     {
//! #         the_value = Some(false)
//! #     }
//! #     the_value
//! # }
//! # use otter_lib::config::Config;
//! # use otter_lib::context::Context;
//! # use otter_lib::dispatch::library::report::{self};
//! let config = Config::default();
//!
//! let mut the_context: Context = Context::from_config(config, None);
//!
//! let not_p_or_q = the_context.clause_from_string("-p q").unwrap();
//! let p_or_not_q = the_context.clause_from_string("p -q").unwrap();
//! let _ = the_context.add_clause(not_p_or_q);
//! let _ = the_context.add_clause(p_or_not_q);
//!
//! assert!(the_context.solve().is_ok());
//! let status = the_context.report();
//! let valuation = the_context.atom_db.valuation_string();
//!
//! assert_eq!(value_of("p", &the_context), Some(false));
//! assert_eq!(value_of("q", &the_context), Some(false));
//!
//! let p_clause = the_context.clause_from_string("p").unwrap();
//! let error = the_context.add_clause(p_clause);
//!
//! the_context.clear_choices();
//!
//! let p_clause = the_context.clause_from_string("p").unwrap();
//! let _p_ok = the_context.add_clause(p_clause);
//!
//! assert_eq!(value_of("p", &the_context), Some(true));
//!
//! assert!(the_context.solve().is_ok());
//!
//! assert_eq!(the_context.report(), report::Solve::Satisfiable);
//! ```

/*
    // Likewise it is not possible to add ¬p ∨ ¬q to 𝐅
    let clause_np_nq = the_context.clause_from_string("-p -q").unwrap();
    assert!(the_context.add_clause(clause_np_nq).is_err());

    assert_eq!(the_context.report(), report::Solve::Satisfiable);

    // todo: update with unit clauses
    println!("The clause database is now:");
    for clause in the_context.clause_db.all_nonunit_clauses() {
        println!("  C {}", clause.as_dimacs(&the_context.atom_db, false))
    }

    // It is possible to add p ∨ q to 𝐅
    let clause_p_q = the_context.clause_from_string("p q").unwrap();
    assert!(the_context.add_clause(clause_p_q).is_ok());
    :?
*/

use crate::{
    context::GenericContext,
    dispatch::{
        library::{
            delta::{self, Delta},
            report::{self, Report},
            stat::Stat,
        },
        macros::{self},
        Dispatch,
    },
    procedures::{
        apply_consequences::{self},
        choice::{self},
    },
    structures::literal::{self},
    types::err::{self},
};

impl<R: rand::Rng + std::default::Default> GenericContext<R> {
    pub fn solve(&mut self) -> Result<report::Solve, err::Context> {
        let total_time = std::time::Instant::now();

        self.preprocess()?;

        'solve_loop: loop {
            self.counters.total_iterations += 1;
            log::trace!("Iteration {}", self.counters.total_iterations);

            self.counters.time = total_time.elapsed();
            let time_limit = self.config.time_limit;
            if time_limit.is_some_and(|limit| self.counters.time > limit) {
                return Ok(report::Solve::TimeUp);
            }

            match self.apply_consequences()? {
                // Non-conflict variants.
                // Note: These variants break or continue the solve loop.
                apply_consequences::Ok::FundamentalConflict => break 'solve_loop,

                apply_consequences::Ok::Exhausted => {
                    //
                    match self.make_choice()? {
                        choice::Ok::Literal(choice) => {
                            self.literal_db.note_choice(choice);
                            self.q_literal(choice)?;
                            continue 'solve_loop;
                        }
                        choice::Ok::Exhausted => break 'solve_loop,
                    }
                }

                // Conflict variants…
                apply_consequences::Ok::UnitClause(literal) => {
                    self.backjump(0);

                    self.q_literal(literal)?;
                }

                apply_consequences::Ok::AssertingClause(key, literal) => {
                    // Safe, as the key is direct from apply_consequences.
                    let the_clause = unsafe { self.clause_db.get_unchecked(&key)? };
                    self.backjump(self.backjump_level(the_clause)?);

                    self.clause_db.note_use(key);
                    macros::send_bcp!(self, Instance, literal, key);

                    self.record_literal(literal, literal::Source::BCP(key));
                    self.q_literal(literal)?;
                }
            }

            self.counters.total_conflicts += 1;
            self.counters.fresh_conflicts += 1;

            if self.luby_fresh_conflict_interrupt() {
                self.counters.luby.next();

                macros::send_stats!(self);

                if self.config.switch.restart {
                    self.backjump(0);
                    self.clause_db.refresh_heap();
                    self.counters.fresh_conflicts = 0;
                    self.counters.restarts += 1;
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
        macros::send_finish!(self);
        Ok(self.report())
    }
}
