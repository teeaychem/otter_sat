//! A queue of observed consequences to be propagated.
//!
//! Observed consequences are atom-value pairs, such  that the given atom *must* have the given value on the current valuation.
//! For convenience, each atom-value pair represented as a literal.
//!
//! ```rust,ignore
//! match context.value_and_queue(CLiteral::new(atom, false), QPosition::Back, 0) {
//!     Ok(Qd) => context.record_literal(literal, literal::Source::Decision),
//!     Ok(Skip) => break,
//!     Err(e) => return Err(e),
//! }
//! ```
//!
//! Queuing a consequence requires specifying:
//! - The atom and it's value, represented as a literal.
//! - Whether to push the consequence to the [front](QPosition::Front) or the [back](QPosition::Back) of the queue.
//! - The decision level at which the consequence was queued.
//!
//! Consequences are queued in various places, such as when adding a unit clause through [add_clause](GenericContext::add_clause).
//! Consequences are applied using [procedures::apply_consequences](crate::procedures::apply_consequences).
//!
//! # Invariants
//!
//! The following invariant is always upheld:
//! <div class="warning">
//! Whenever the valuation is extended so that atom <i>a</i> has value <i>v</i>, that atom <i>a</i> has value <i>v</i> is added to the consequence queue.
//! </div>
//!
//! # Details
//!
//! In order to help uphold the given invariant, queuing a literal results in an immediate attempt to update the current valuation with the observation.
//! So, it is sufficient to push to the queue in order to update the valuation.
//! - If the consequence is *already* part of the current valuation, nothing happens.\
//!   In this case, given the invariant above conseqence is, or has already been, on the queue.
//! - If the consequence is *not* already part of the current valuation, the valuation is updated with the consequence and a literal representing the atom-value pair is added to the queue, ready to be examined by a process such as [BCP](crate::procedures::bcp).
//! - If the consequence *conflicts* with the current valuation, a conflict has been found and an error is returned.\
//!   Here, a prodedure such as [analysis](crate::procedures::analysis) may be used to recover from the conflict.
//!
//! Interaction with the queue as a [std::collections::VecDeque] is preferred, though further methods may be attached to other structs.
//! For example, [GenericContext::clear_q] provides a convenient way to clear all consequences from a given level.
//!
//! # Consequence delay
//!
//! The intended use of the consequence queue is to allow for the decision that a atom *will* have, or the observation that an atom *must* have, some value to be used to update the valuation immediately, and for the task of examining the consequences of this to be delayed.
//!
//! This is particularly useful to avoid multiple passes as updating the watch literals for a clause, as multiple candidate watch literals at the time of the queuing may be ruled out by the time the consequence is applied.
//!
//! Further, as a conflict requires immediate backjumping, this use may avoid redundant propagation from consequences queued when a conflict is found --- though, it may be that applying those consequences would have led to a different (and perhaps more useful) learned clause.

use std::borrow::Borrow;

use crate::{
    context::GenericContext,
    db::DecisionLevelIndex,
    misc::log::targets::{self},
    structures::literal::{CLiteral, Literal},
    types::err::{self},
};

/// A queue of observed consequences and the level at which the consequence was observed.
pub type ConsequenceQ = std::collections::VecDeque<(CLiteral, DecisionLevelIndex)>;

/// Possible 'Ok' results of queuing a literal.
pub enum ConsequenceQueueOk {
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
    pub fn clear_q(&mut self, from: DecisionLevelIndex) {
        self.consequence_q.retain(|(_, c)| *c <= from);
    }

    /// Assigns the given value to the given atom, if possible, and places the pair (represented as a literal) on the consequence queue.
    /// Otherwise, returns an error.
    ///
    /// A literal can be queued so long as it does not conflict with the current valuation.
    /// ```rust,ignore
    /// context.value_and_queue(CLiteral::new(atom, value));
    /// ```
    pub fn value_and_queue(
        &mut self,
        literal: impl Borrow<CLiteral>,
        position: QPosition,
        level: DecisionLevelIndex,
    ) -> Result<ConsequenceQueueOk, err::ConsequenceQueueError> {
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
                Ok(ConsequenceQueueOk::Qd)
            }
            Ok(_) => Ok(ConsequenceQueueOk::Skip),
            Err(_) => {
                log::trace!(target: targets::QUEUE, "Queueing {} failed.", literal.borrow());
                Err(err::ConsequenceQueueError::Conflict)
            }
        }
    }

    /// Places an atom-value (represented as a literal) consequence on the consequence queue, always.
    ///
    /// # Soundness
    /// This does not check to ensure the literal is not (already) unsatisfiable on the current valuation.
    /// I.e., that it is not possible to value the atom of the literal with the polarity of the literal.
    /// [GenericContext::value_and_queue] may be appropriate.
    pub fn push_to_consequence_queue(
        &mut self,
        literal: impl Borrow<CLiteral>,
        level: DecisionLevelIndex,
        position: QPosition,
    ) {
        match position {
            QPosition::Front => self.consequence_q.push_front((*literal.borrow(), level)),
            QPosition::Back => self.consequence_q.push_back((*literal.borrow(), level)),
        }
    }
}
