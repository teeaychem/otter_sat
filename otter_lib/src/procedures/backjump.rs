//! Recovery from a conflict.
//!
//! # Overview
//!
//! A backjump is a 'jump' from some (higher) choice level to some previous (lower) choice level.
//!
//! Typically, a backjump is made from level *l* to level *l - i* because a conflict was found at level *l* and analysis produced a clause which asserts some literal at level *l - i*.
//! In this case, all choices and all consequences of those choices from level *l* down to level *l - i* are undone, and any queued consequences of the choice are removed from the consequence queue.
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
//! # [backjump_level](GenericContext::backjump_level) --- The backjump level of a(n inconsistent) clause
//!
//! The backjump level of a clause is the highest level for which the clause is consistent on the corresponding valuation.
//!
//! This definition is partial, in that a clause may be inconsistent without a decision having been made.
//! Though, in this case there is no need for a backjump level, as the formula itself must be inconsistent and there is no need to continue a solve.
//!
//! - Soundness
//!   + With respect to implementation, the backjump level of a clause is the second highest choice index from the given literals, if more than two choices have been made, and 0 (zero) otherwise. \
//!     In this respect the implementation of [backjump_level](GenericContext::backjump_level) is only sound to use when applied to an clause inconsistent with the current valuation.
//!
//! # Example
//!
//! ```rust,ignore
//! if let AssertingClause(key, literal) = result {
//!     let the_clause = self.clause_db.get(&key)?;
//!     let index = self.backjump_level(the_clause)?;
//!     self.backjump(index);
//! }
//! ```

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
    pub fn backjump(&mut self, target_level: LevelIndex) {
        // log::trace!(target: crate::log::targets::BACKJUMP, "Backjump from {} to {}", self.levels.index(), to);

        // Sufficiently safe:
        // The pop from the choice stack is fine, as choice_count is the height of the choice stack.
        // So, the elements to pop must exist.
        // And, if an atom is in the choice stack is should certainly be in the atom database.
        unsafe {
            for _ in 0..(self.literal_db.choice_count().saturating_sub(target_level)) {
                self.atom_db
                    .drop_value(self.literal_db.last_choice_unchecked().atom());
                for (_, literal) in self.literal_db.last_consequences_unchecked() {
                    self.atom_db.drop_value(literal.atom());
                }
            }
            self.literal_db.forget_last_choice();
        }
        self.clear_q(target_level);
    }

    /// The bacjump level of a unsatisfiable clause.
    ///
    /// For documentation, see [procedures::backjump](crate::procedures::backjump).
    // Work through the clause, keeping an ordered record of the top two decision levels: (second_to_top, top)
    pub fn backjump_level(&self, clause: &impl Clause) -> Result<LevelIndex, err::Context> {
        match clause.size() {
            0 => panic!("!"),
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
