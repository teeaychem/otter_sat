//! A queue of observed consequences to be propagated.
//!
//! Observed consequences are atom-value pairs, such  that the given atom *must* have the given value on the current valuation.
//! For convenience, each atom-value pair represented as a literal.
//!
//! The following invariant is always upheld:
//! <div class="warning">
//! Whenever the current valuation is extended so that atom <i>a</i> has value <i>v</i>, that atom <i>a</i> has value <i>v</i> is added to the consequence queue.
//! </div>
//!
//! Queuing a literal results in an immediate attempt to update the current valuation with the observation.
//! - If the consequence is *already* part of the current valuation, nothing happens.\
//!   In this case, given the invariant above conseqence is, or has already been, on the queue.
//! - If the consequence is *not* already part of the current valuation, the valuation is updated with the consequence and a literal representing the atom-value pair is added to the queue, ready to be examined by a process such as [BCP](crate::procedures::bcp).
//! - If the consequence *conflicts* with the current valuation, a conflict has been found and an error is returned.\
//!   Here, a prodedure such as [analysis](crate::procedures::analysis) may be used to recover from the conflict.
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

    /// The literal was skipped.
    ///
    /// This may happen, e.g., if the consequences of the literal are (set to be) applied.
    Skip,
}

/// Relative positions to place a literal on the consequence queue.
pub enum QPosition {
    /// The front of the queue
    Front,

    /// The back of the queue
    Back,
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
        self.consequence_q.retain(|(_, c)| *c <= from);
    }

    /// Queues a literal, if possible. Otherwise, returns an error.
    ///
    /// A literal can be queued so long as it does not conflict with the current valuation.
    /// ```rust,ignore
    /// let a_literal = abLiteral::fresh(atom, value);
    /// self.q_literal(a_literal);
    /// ```
    pub fn q_literal(
        &mut self,
        literal: impl Borrow<abLiteral>,
        position: QPosition,
        level: LevelIndex,
    ) -> Result<Ok, err::Queue> {
        let valuation_result = unsafe {
            self.atom_db.set_value(
                literal.borrow().atom(),
                literal.borrow().polarity(),
                Some(level),
            )
        };
        match valuation_result {
            Ok(super::atom::AtomValue::NotSet) => {
                // TODO: improvements?
                match position {
                    QPosition::Front => self.consequence_q.push_front((*literal.borrow(), level)),
                    QPosition::Back => self.consequence_q.push_back((*literal.borrow(), level)),
                }
                log::trace!(target: targets::QUEUE, "Queued {} at level {level}.", literal.borrow());
                Ok(Ok::Qd)
            }
            Ok(_) => Ok(Ok::Skip),
            Err(_) => {
                log::trace!(target: targets::QUEUE, "Queueing {} failed.", literal.borrow());
                Err(err::Queue::Conflict)
            }
        }
    }

    /// Places a literal on the consequence queue, always.
    ///
    /// # Soundness
    /// This does not check to ensure the literal is not (already) unsatisfiable on the current valuation.
    /// I.e., that it is not possible to value the atom of the literal with the polarity of the literal.
    /// [GenericContext::q_literal] may be appropriate.
    pub fn q_literal_regardless(
        &mut self,
        literal: impl Borrow<abLiteral>,
        level: LevelIndex,
        position: QPosition,
    ) {
        match position {
            QPosition::Front => self.consequence_q.push_front((*literal.borrow(), level)),
            QPosition::Back => self.consequence_q.push_back((*literal.borrow(), level)),
        }
    }
}
