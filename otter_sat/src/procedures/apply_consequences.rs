//! Applies queued consequences.
//!
//! For an overview of apply_consequence within a solve, see the documentation of the [solve procedure](crate::procedures::solve).
//!
//! Roughly, apply_consequences implements an instance of the operator which:
//!
//! - Returns *unsatisfiable*, if it is not possible to apply the consequence relation.
//! - Returns *satisfiable*, if the formula entails itself and the valuation is complete.
//! - Makes a decision, if the formula entails itself and the valuation is partial.
//! - Backjumps to a different valuation, if the formula entails some formula with an additional clause.
//!
//! - A return of *unsatisfiable* is represented as a `FundamentalConflict`.
//! - A return of a new clause is represented with a [key](crate::db::ClauseKey) to the clause, and an asserted literal.
//! - No change is represented by a return of `Exhausted`.
//!   + It is up to a caller of apply_consequences to note whether the background valuation is complete.
//!
//! The following invariant is upheld:
//! <div class="warning">
//! apply_consequences returns the same formula only if there are no further consequences to apply.
//! </div>
//!
//! # Overview
//!
//! apply_consequences
//!
//! At a high level [apply_consequences](GenericContext::apply_consequences) sequences a handful of more basic procedures in a loop:
//! - Take a queued consequence.
//! - Apply boolean constraint propagation with respect to the consequence.
//! - If no conflict is found, continue.
//! - Otherwise, perform conflict analysis and break.
//!
//! These procedures are sequenced as a single procedure as the procedure may loop until inconsistency of the formula is established, a consistent valuation is found, or some decision needs to be made in order to progress.
//! Though, in practice [apply_consequences](GenericContext::apply_consequences) returns at the first conflict found.
//! This is to allow for further actions to be taken due to a conflict having been found.
//!
//! ```rust,ignore
//! while let Some((literal, _)) = self.consequence_q.front() {
//!     match self.bcp(literal) {
//!         Ok(()) => self.consequence_q.pop_front(), // continue applying consequences
//!         Err(err::BCP::Conflict(key)) => {
//!             if !self.literal_db.decision_made() {
//!                 return Ok(Ok::FundamentalConflict);
//!             }
//!
//!             match self.conflict_analysis(&key)? {
//!                 // Analysis is only called when some decision has been made.
//!                 analysis::Ok::FundamentalConflict => !,
//!
//!                 analysis::Ok::MissedPropagation {
//!                     clause_key: key,
//!                     asserted_literal: literal,
//!                 } => {
//!                     // return and complete the instance of propagation
//!                     ...
//!                     continue 'application;
//!                 }
//!
//!                 analysis::Ok::UnitClause(key) => {
//!                     return Ok(Ok::UnitClause(key));
//!                 }
//!
//!                 analysis::Ok::AssertingClause {
//!                     clause_key: key,
//!                     asserted_literal: literal,
//!                 } => {
//!                     return Ok(Ok::AssertingClause(key, literal));
//!                 }
//!             }
//!         }
//!     }
//! }
//! Ok(Ok::Exhausted)
//! ```
//!
//! # Missed propagations
//!
//! In some situations the opportunity to propagate a consequence may be 'missed'.
//! This is identified when conflict analysis returns a clause already present in the clause database.
//!
//! As missed propagation implies BCP does not propagate *all* boolean constraints before identifying a conflict.
//! For, a missed propagation means it is possible to backjump to some sub-valuation on which the clause is asserting, and the valuation obtained by cohering with the asserted literal must be different from the valuation on which the clause is unsatisfiable.
//!
//! Note, an unsatisfiable formula is unsatisfiable regardless of whether any propagation are missed.
//! And, a satisfiable formula is satisfiable so long all original clause propagations are made.
//! So, a solver may be sound and miss some propagations.
//!
//! Regardless, missed propagations are returned to and their consequences applied *within* an instance of apply_consequences, in order to maintain the invariant that apply_consequences returns the same formula only if there are no further consequences to apply.

use crate::{
    context::GenericContext,
    db::{dbStatus, ClauseKey},
    dispatch::{
        library::delta::{self, Delta},
        macros::{self},
        Dispatch,
    },
    procedures::analysis,
    structures::literal::{self, abLiteral},
    types::err,
};

/// Ok results of apply_consequences.
pub enum Ok {
    /// A conflict was found, and so the formula is unsatisfiable.
    FundamentalConflict,

    /// A unit clause was derived from
    UnitClause(abLiteral),
    AssertingClause(ClauseKey, abLiteral),
    Exhausted,
}

impl<R: rand::Rng + std::default::Default> GenericContext<R> {
    /// Applies queued consequences.
    ///
    /// apply_consequences applies BCP to the consequence queue until either a conflict is found or the queue is exhausted.
    ///
    /// Queued consequences are removed from the queue only if BCP was successful.
    /// For, in the case of a conflict the consequence may remain, and otherwise will be removed from the queue during a backjump.
    pub fn apply_consequences(&mut self) -> Result<Ok, err::Context> {
        use crate::db::consequence_q::QPosition::{self};

        'application: loop {
            let Some((literal, _)) = self.consequence_q.front().cloned() else {
                return Ok(Ok::Exhausted);
            };

            match unsafe { self.bcp(literal) } {
                Ok(()) => {
                    self.consequence_q.pop_front();
                }
                Err(err::BCP::CorruptWatch) => return Err(err::Context::BCP),
                Err(err::BCP::Conflict(key)) => {
                    //
                    if !self.literal_db.decision_made() {
                        self.status = dbStatus::Inconsistent;

                        macros::send_atom_db_delta!(self, delta::AtomDB::Unsatisfiable(key));

                        return Ok(Ok::FundamentalConflict);
                    }

                    match self.conflict_analysis(&key)? {
                        // Analysis is only called when some decision has been made.
                        analysis::Ok::FundamentalConflict => panic!("!"),

                        analysis::Ok::MissedPropagation {
                            key,
                            literal: asserted_literal,
                        } => {
                            // panic!("!");
                            let the_clause = unsafe { self.clause_db.get_unchecked(&key)? };

                            let index = self.non_chronological_backjump_level(the_clause)?;
                            self.backjump(index);

                            self.q_literal(asserted_literal, QPosition::Front)?;

                            macros::send_bcp_delta!(self, Instance, asserted_literal, key);

                            self.record_literal(asserted_literal, literal::Source::BCP(key));

                            continue 'application;
                        }

                        analysis::Ok::UnitClause(key) => {
                            return Ok(Ok::UnitClause(key));
                        }

                        analysis::Ok::AssertingClause { key, literal } => {
                            return Ok(Ok::AssertingClause(key, literal));
                        }
                    }
                }
            }
        }
    }
}
