//! A queue of observed consequences to be propagated.
//!
//! For primary use case see the following associated [GenericContext] methods:
//! - [GenericContext::q_literal]
//!
//! Interaction with the queue as a [std::collections::VecDeque] is preferred, though further methods may be attached to other structs.
//! For example, [GenericContext::clear_q] provides a convenient way to clear all consequences from a given level.

use std::borrow::Borrow;

use crate::{
    context::GenericContext,
    db::LevelIndex,
    misc::log::targets::{self},
    structures::literal::{abLiteral, Literal},
    types::err::{self},
};

/// A queue of observed consequences and the level at which the consequence was observed.
pub type ConsequenceQ = std::collections::VecDeque<(abLiteral, LevelIndex)>;

/// Possible 'Ok' results of queuing a literal.
pub enum Ok {
    /// The literal was (successfully) queued.
    Qd,
}

impl<R: rand::Rng + std::default::Default> GenericContext<R> {
    /// Clears all queued consequences from the given level index up to the current level index.
    /// ```rust,ignore
    /// pub fn backjump(&mut self, to: LevelIndex) {
    ///     ...
    ///     self.clear_consequences(to);
    /// }
    /// ```
    pub fn clear_q(&mut self, from: LevelIndex) {
        self.consequence_q.retain(|(_, c)| *c < from);
    }

    /// Queues a literal, if possible. Otherwise, returns an error.
    ///
    /// A literal can be queued so long as it does not conflict with the current valuation.
    /// ```rust,ignore
    /// let a_literal = abLiteral::fresh(atom, value);
    /// self.q_literal(a_literal);
    /// ```
    pub fn q_literal(&mut self, literal: impl Borrow<abLiteral>) -> Result<Ok, err::Queue> {
        let valuation_result = unsafe {
            self.atom_db.set_value(
                literal.borrow().atom(),
                literal.borrow().polarity(),
                Some(self.literal_db.choice_count()),
            )
        };
        match valuation_result {
            Ok(_) => {
                // TODO: improvements?
                self.consequence_q
                    .push_back((*literal.borrow(), self.literal_db.choice_count()));

                Ok(Ok::Qd)
            }
            Err(_) => {
                log::trace!(target: targets::QUEUE, "Queueing {} failed.", literal.borrow());
                Err(err::Queue::Conflict)
            }
        }
    }
}
