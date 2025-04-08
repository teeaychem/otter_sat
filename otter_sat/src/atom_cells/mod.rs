/*!
A database of assignment related objects.

# Assignemnts

Assignments are atom-value pairs with a source, such  that the given atom *must* have the given value on the current valuation.
For convenience, each atom-value pair represented as a literal.

The following invariant is always upheld:
<div class="warning">
Whenever the valuation is extended so that atom <i>a</i> has value <i>v</i>, that atom <i>a</i> has value <i>v</i> is added to the list of assignments, togeter with the source of that assignment.
</div>

# Details

In order to help uphold the given invariant, a check on the value of a literal should be made prior to recording an assignment.
So, it is sufficient to push to the queue in order to update the valuation.
- If the assignment is *already* part of the current valuation, nothing should happen.\
  In this case, given the invariant above conseqence is, or has already been, on the queue.
- If the consequence is *not* already part of the current valuation, the valuation is updated with the consequence and a literal representing the atom-value pair is added, ready to be examined by a process such as [BCP](crate::procedures::bcp).
- If the consequence *conflicts* with the current valuation, a conflict has been found and an error is returned.\
  Here, a prodedure such as [analysis](crate::procedures::analysis) may be used to recover from the conflict.

## Queued propagations

A queue of observed consequences to be propagated is identified by `q_head`.
If the head points to some index in the list of assignments, then that assignment and all those assignments after are yet to be propagated.
Otherwise, the queue head exceeds the assignment count by an offset of one and automatically points to any fresh assignment.
(Note, the queue head is adjusted when backjumping, if required.)

Consequences are queued in various places, such as when adding a unit clause through [add_clause](crate::context::GenericContext::add_clause).
Consequences are applied using [procedures::apply_consequences](crate::procedures::apply_consequences).

### Consequence delay

The intended use of the consequence queue is to allow for the decision that a atom *will* have, or the observation that an atom *must* have, some value to be used to update the valuation immediately, and for the task of examining the consequences of this to be delayed.

This is particularly useful to avoid multiple passes as updating the watch literals for a clause, as multiple candidate watch literals at the time of the queuing may be ruled out by the time the consequence is applied.

Further, as a conflict requires immediate backjumping, this use may avoid redundant propagation from consequences queued when a conflict is found --- though, it may be that applying those consequences would have led to a different (and perhaps more useful) learned clause.

*/

use std::collections::HashSet;

use cell::Cell;
use config::BufferConfig;

use crate::{
    context::callbacks::CallbackOnPremises,
    db::{ClauseKey, LevelIndex},
    structures::{atom::Atom, literal::CLiteral},
};
#[doc(hidden)]
mod cell;

pub mod config;
#[doc(hidden)]
pub mod methods;

#[doc(hidden)]
pub mod valuation;

/// Possilbe 'Ok' results from resolution using a resolution buffer.
pub enum ResolutionOk {
    /// A unique implication point was identified.
    UIP,

    /// Resolution produced a unit clause.
    UnitClause,

    /// Resolution identified a clause already in the database.
    Repeat(ClauseKey, CLiteral),
}

/// A buffer for use when applying resolution to a sequence of clauses.
pub struct AtomCells {
    /// A count of literals in the clause whose atoms do not have a value on the given interpretation.
    valueless_count: usize,

    /// The length of the clause.
    clause_length: usize,

    /// The (direct) premises used top derive the clause.
    premises: HashSet<ClauseKey>,

    /// The buffer.
    pub buffer: Vec<Cell>,

    /// A stack of modified atoms, with the original value stored as literal polarity.
    merged_atoms: Vec<Atom>,

    /// A (typically derived) configuration for the instance of resolution.
    config: BufferConfig,

    /// The callback used on completion
    callback_premises: Option<Box<CallbackOnPremises>>,
}

impl AtomCells {
    /// Set the callback made when an instance of resolution completes and returns premises used to `callback`.
    pub fn set_callback_resolution_premises(&mut self, callback: Box<CallbackOnPremises>) {
        self.callback_premises = Some(callback);
    }

    /// Make the callback requested when an instance of resolution completes and returns premises used, if defined.
    pub fn make_callback_resolution_premises(&mut self, premises: &HashSet<ClauseKey>) {
        if let Some(callback) = &mut self.callback_premises {
            callback(premises);
        }
    }

    /// Which decision an atom was valued on.
    pub fn level(&self, atom: Atom) -> Option<LevelIndex> {
        self.get(atom).level
    }

    /// Returns the '*previous*' value of the atom from the valuation stored in the [AtomDB].
    ///
    /// When a context is built this value may be random.
    pub fn previous_value_of(&self, atom: Atom) -> bool {
        self.get(atom).previous_value
    }

    pub fn get(&self, atom: Atom) -> &Cell {
        // # Safety
        // A cell is created together with the addition of an atom
        unsafe { self.buffer.get_unchecked(atom as usize) }
    }

    pub fn get_mut(&mut self, atom: Atom) -> &mut Cell {
        // # Safety
        // A cell is created together with the addition of an atom
        unsafe { self.buffer.get_unchecked_mut(atom as usize) }
    }
}
