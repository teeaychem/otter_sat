//! Applies queued consequences.
//!
//! Applying queued consequences results in a consistent database or a clause to avoid discovered inconsistency.
//!
//! At a high level [apply_consequences](GenericContext::apply_consequences) sequences a handful of more basic procedures in a loop:
//! - Take a queued consequence
//! - Apply boolean constraint propagation with respect to the consequence.
//! - If no conflict is found, continue.
//! - Otherwise, perform conflict analysis and break.
//!
//! These procedures are sequenced as a single procedure as the procedure may loop until inconsistency of the formula is established, a consistent valuation is found, or some choice needs to be made in order to progress.
//! Though, in practice [apply_consequences](GenericContext::apply_consequences) returns at the first conflict found.
//! This is to allow for further actions to be taken due to a conflict having been found.
//!
//! Still, in the above respect, [apply_consequences](GenericContext::apply_consequences) is purely deductive, and can be seen as instantiating a purely tautological consequence relation which entails:
//! - A (tautological) consequence of the formula with some additional clause, if a conflict is found.
//! - The current formula itself, if some choice needs to be made before conflicts are found.
//! - Top, if a consistent valuation is found.
//!
//! # Example
//!
//! ```rust,ignore
//! match self.apply_consequences()? {
//!     Ok::FundamentalConflict => ...,
//!
//!     Ok::UnitClause(key) => {
//!         self.backjump(0);
//!         ...
//!     }
//!
//!     Ok::AssertingClause(key, literal) => {
//!         let the_clause = self.clause_db.get_db_clause(&key)?;
//!         let index = self.backjump_level(the_clause)?;
//!         self.backjump(index);
//!         ...
//!     }
//!
//!     Ok::Exhausted => {
//!         match self.make_choice()? {
//!             choice::Ok::Made => ...,
//!             choice::Ok::Exhausted => ...,
//!         }
//!     }
//! }
//! ```
//!
//! # Notes
//! - appy_consequences breaks on conflicts, but does not break on missed implications.

use crate::{
    context::GenericContext,
    db::{dbStatus, ClauseKey},
    dispatch::{library::delta, Dispatch},
    procedures::analysis,
    structures::literal::{self, abLiteral},
    types::err,
};

/// Ok results of apply_consequences.
pub enum Ok {
    /// A conflict was found, and so the formula is unsatisfiable.
    FundamentalConflict,

    /// A unit clause was derived from
    UnitClause(ClauseKey),
    AssertingClause(ClauseKey, abLiteral),
    Exhausted,
}

impl<R: rand::Rng + std::default::Default> GenericContext<R> {
    /// Expand queued consequences:
    /// Performs an analysis on apparent conflict.
    pub fn apply_consequences(&mut self) -> Result<Ok, err::Context> {
        'application: while let Some((literal, _)) = self.consequence_q.pop_front() {
            match unsafe { self.bcp(literal) } {
                Ok(()) => {}
                Err(err::BCP::CorruptWatch) => return Err(err::Context::BCP),
                Err(err::BCP::Conflict(key)) => {
                    //
                    if !self.literal_db.choice_made() {
                        self.status = dbStatus::Inconsistent;

                        if let Some(dispatcher) = &self.dispatcher {
                            let delta = delta::AtomDB::Unsatisfiable(key);
                            dispatcher(Dispatch::Delta(delta::Delta::AtomDB(delta)));
                        }

                        return Ok(Ok::FundamentalConflict);
                    }

                    match self.conflict_analysis(&key)? {
                        // Analysis is only called when some decision has been made.
                        analysis::Ok::FundamentalConflict => panic!("!"),

                        analysis::Ok::MissedPropagation {
                            clause_key: key,
                            asserted_literal: literal,
                        } => {
                            let the_clause = unsafe { self.clause_db.get_unchecked(&key)? };

                            let index = self.backjump_level(the_clause)?;
                            self.backjump(index);

                            self.q_literal(literal)?;

                            if let Some(dispatcher) = &self.dispatcher {
                                let delta = delta::BCP::Instance {
                                    clause: key,
                                    literal,
                                };
                                dispatcher(Dispatch::Delta(delta::Delta::BCP(delta)));
                            }
                            self.record_literal(literal, literal::Source::BCP(key));

                            continue 'application;
                        }

                        analysis::Ok::UnitClause(key) => {
                            return Ok(Ok::UnitClause(key));
                        }

                        analysis::Ok::AssertingClause {
                            clause_key: key,
                            asserted_literal: literal,
                        } => {
                            return Ok(Ok::AssertingClause(key, literal));
                        }
                    }
                }
            }
        }
        Ok(Ok::Exhausted)
    }
}
