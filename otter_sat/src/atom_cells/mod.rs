/*!
A database of assignment related objects.

# Assignments

Assignments are atom-value pairs with a source, such  that the given atom *must* have the given value on the current valuation.
For convenience, each atom-value pair represented as a literal.

The following invariant is always upheld:
<div class="warning">
Whenever the valuation is extended so that atom <i>a</i> has value <i>v</i>, that atom <i>a</i> has value <i>v</i> is added to the list of assignments, together with the source of that assignment.
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

use cell::AtomCell;

use crate::{
    context::callbacks::CallbackOnPremises,
    db::{ClauseKey, LevelIndex},
    structures::{atom::Atom, literal::CLiteral},
};
#[doc(hidden)]
pub mod cell;

#[doc(hidden)]
pub mod methods;

#[doc(hidden)]
pub mod valuation;

#[doc(hidden)]
pub mod resolution;

/// Possible 'Ok' results from resolution using a resolution buffer.
pub enum ResolutionOk {
    /// A unique implication point was identified.
    UIP,

    /// Resolution produced a unit clause.
    UnitClause,

    /// Resolution identified a clause already in the database.
    Repeat(ClauseKey, CLiteral),
}

/// A store of information to return to during recursive minimization.
struct ReMiTodo {
    /// The key to the clause under consideration.
    pub key: ClauseKey,

    /// The index of the literal in the clause to next consider (or the length of the clause to indicate exhaustion).fb
    pub index: usize,
}

/// A buffer for use when applying resolution to a sequence of clauses.
pub struct AtomCells {
    /// A count of literals in the clause whose atoms do not have a value on the given interpretation.
    valueless_count: usize,

    /// The length of the clause.
    clause_length: usize,

    /// The (direct) premises used top derive the clause.
    premises: HashSet<ClauseKey>,

    /// The cells.
    pub cells: Vec<AtomCell>,

    /// A stack of modified atoms, with the original value stored as literal polarity.
    /// Note, merged atoms are cleared when finialising a learnt clause.
    merged_atoms: Vec<Atom>,

    /// The callback used on completion
    callback_premises: Option<Box<CallbackOnPremises>>,

    /// A persistent stack used during recursive minimization.
    recursive_minimization_todo: Vec<ReMiTodo>,

    /// Reset to CellStatus::Valuation after checking failed literals.
    cached_removable_status_atoms: Vec<Atom>,
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
        self.get_cell(atom).level
    }

    /// Returns the '*previous*' value of the atom.
    ///
    /// When a context is built this value may be random.
    pub fn previous_value_of(&self, atom: Atom) -> bool {
        self.get_cell(atom).previous_value
    }

    /// The the cell for 'atom'.
    pub fn get_cell(&self, atom: Atom) -> &AtomCell {
        // # Safety: A cell is created together with the addition of an atom
        unsafe { self.cells.get_unchecked(atom as usize) }
    }

    /// The the cell for 'atom', mutable.
    pub fn get_cell_mut(&mut self, atom: Atom) -> &mut AtomCell {
        // # Safety: A cell is created together with the addition of an atom
        unsafe { self.cells.get_unchecked_mut(atom as usize) }
    }
}
