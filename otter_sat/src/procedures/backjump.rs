//! Recovery from a conflict.
//!
//! # Overview
//!
//! A backjump is a 'jump' from some (higher) decision level to some previous (lower) decision level.
//!
//! Typically, a backjump is made from level *l* to level *l - i* because a conflict was found at level *l* and analysis produced a clause which asserts some literal at level *l - i*.
//! In this case, all decisions and all consequences of those decisions from level *l* down to level *l - i* are undone, and any queued consequences of the decision are removed from the consequence queue.
//!
//! # Methods
//!
//! # [backjump](GenericContext::backjump) --- Backjump to a target level
//!
//! Performs a backjump to some level.
//!
//! For sound application the target level must be equal to or lower than the current level.
//! Still, passing a traget level greater than the current level is safe --- nothing will happen.
//!
//! # [backjump_level](GenericContext::non_chronological_backjump_level) --- The backjump level of a(n unsatisfiable) clause
//!
//! The backjump level of a clause is the highest level for which the clause is satisfiable on the corresponding valuation.
//!
//! This definition is partial, in that a clause may be unsatisfiable without a decision having been made.
//! Though, in this case there is no need for a backjump level, as the formula itself must be unsatisfiable and there is no need to continue a solve.
//!
//! - Soundness
//!   + With respect to implementation, the backjump level of a clause is the second highest decision index from the given literals, if more than two decisions have been made, and 0 (zero) otherwise. \
//!     In this respect the implementation of non_chronological_backjump_level is only sound to use when applied to an clause unsatisfiable on the current valuation.
//!
//! # Example
//!
//! ```rust,ignore
//! if let AssertingClause(key, literal) = result {
//!     let the_clause = self.clause_db.get(&key)?;
//!     let index = self.non_chronological_backjump_level(the_clause)?;
//!     self.backjump(index);
//! }
//! ```
//!
//! # Literature
//!
//! See [Chronological Backtracking](https://doi.org/10.1007/978-3-319-94144-8_7) for a discussion of chronological and non-chronological backjumping --- and a follow-up: [Backing Backtracking](https://www.doi.org/10.1007/978-3-030-24258-9_18).

use std::cmp;

use crate::{
    context::GenericContext,
    db::LevelIndex,
    misc::log::targets::{self},
    structures::{clause::Clause, literal::Literal},
    types::err,
};

impl<R: rand::Rng + std::default::Default> GenericContext<R> {
    /// Backjumps to the given target level.
    ///
    /// For documentation, see [procedures::backjump](crate::procedures::backjump).
    pub fn backjump(&mut self, target: LevelIndex) {
        // log::trace!(target: crate::log::targets::BACKJUMP, "Backjump from {} to {}", self.levels.index(), to);

        // Safety:
        // The pop from the decision stack is fine, as decision_count is the height of the decision stack.
        // So, the elements to pop must exist.
        // And, if an atom is in the decision stack is should certainly be in the atom database.
        unsafe {
            for _ in 0..(self.literal_db.current_level().saturating_sub(target)) {
                self.atom_db
                    .drop_value(self.literal_db.top_decision_unchecked().atom());
                for consequence in self.literal_db.top_consequences_unchecked() {
                    self.atom_db.drop_value(consequence.atom());
                }
                self.literal_db.forget_top_level();
            }
        }
        self.clear_q(target);
    }

    /// The non-chronological backjump level of a unsatisfiable clause.
    ///
    /// + The *non-chronological* backjump level is the previous decision level of a clause.
    /// + The *chronological* backjump level is the previous decision level of a context.
    ///
    /// For documentation, see [procedures::backjump](crate::procedures::backjump).
    pub fn non_chronological_backjump_level(
        &self,
        clause: &impl Clause,
    ) -> Result<LevelIndex, err::ErrorKind> {
        match clause.size() {
            0 => {
                panic!("! Attempted search for non-chronological backjump level on an empty clause")
            }
            1 => Ok(self.literal_db.lowest_decision_level()),
            _ => {
                // Work through the clause, keeping an ordered record of the top two decision levels: (second_to_top, top)
                let mut top_two = (None, None);
                for literal in clause.literals() {
                    let Some(dl) =
                        (unsafe { self.atom_db.atom_decision_level_unchecked(literal.atom()) })
                    else {
                        log::error!(target: targets::BACKJUMP, "{literal} was not chosen");
                        return Err(err::ErrorKind::Backjump);
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
                    (None, _) => Ok(self.literal_db.lowest_decision_level()),
                    (Some(second_to_top), _) => Ok(cmp::max(
                        self.literal_db.lowest_decision_level(),
                        second_to_top,
                    )),
                }
            }
        }
    }

    /// Removes assumptions from a context by unbinding the value from any atom bound due to an assumption.
    pub fn clear_assumptions(&mut self) {
        if !self.literal_db.config.stacked_assumptions {
            for assumption in self.literal_db.stored_assumptions() {
                unsafe { self.atom_db.drop_value(assumption.atom()) };
            }
        }

        self.literal_db.lowest_decision_level = 0;
        self.literal_db.clear_assumptions();
    }
}
