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

#[doc(hidden)]
pub mod activity;
#[doc(hidden)]
pub mod valuation;
pub mod watch_db;

use std::borrow::Borrow;

use crate::{
    config::{Activity, Config, dbs::AtomDBConfig},
    db::LevelIndex,
    generic::index_heap::IndexHeap,
    misc::log::targets::{self},
    structures::{
        atom::Atom,
        literal::{CLiteral, Literal},
        valuation::{Valuation, vValuation},
    },
};

#[derive(Default)]
pub struct Trail {
    pub literals: Vec<CLiteral>,
    pub level_indicies: Vec<usize>,
    pub q_head: usize,
    pub initial_decision_level: LevelIndex,
}

use super::watches::Watches;

/// The atom database.
pub struct AtomDB {
    /// A current (often partial) [valuation](Valuation).
    pub valuation: vValuation,

    /// The previous (often partial) [valuation](Valuation) (or some randomised valuation).
    pub previous_valuation: Vec<bool>,

    /// An [IndexHeap] recording the activty of atoms, where any atom without a value is 'active' on the heap.
    pub activity_heap: IndexHeap<Activity>,

    /// A map from atoms to levels.
    pub atom_level_map: Vec<Option<LevelIndex>>,

    /// The assignments made, in order from initial to most recent.
    pub trail: Trail,

    /// A local configuration, typically derived from the configuration of a context.
    pub config: AtomDBConfig,
}

#[derive(Debug, PartialEq, Eq)]
/// The status of the valuation of an atom, relative to some known valuation or literal.
pub enum AtomValue {
    /// The atom has no value.
    NotSet,

    /// The value of the atoms is the same as the known valuation, or polarity of the literal.
    Same,

    /// The value of the atoms is not the same as the known valuation, or polarity of the literal.
    Different,
}

impl AtomDB {
    /// A new [AtomDB] with local configuration options derived from `config`.
    pub fn new(config: &Config) -> Self {
        AtomDB {
            activity_heap: IndexHeap::default(),

            valuation: Vec::default(),
            previous_valuation: Vec::default(),
            atom_level_map: Vec::default(),

            trail: Trail::default(),

            config: config.atom_db.clone(),
        }
    }

    /// A count of atoms in the [AtomDB].
    pub fn count(&self) -> usize {
        self.valuation.len()
    }

    /// The current valuation, as some struction which implements the valuation trait.
    pub fn valuation(&self) -> &impl Valuation {
        &self.valuation
    }

    /// The current valuation, as a canonical [vValuation].
    pub fn valuation_canonical(&self) -> &vValuation {
        &self.valuation
    }

    /// Which decision an atom was valued on.
    ///
    /// # Safety
    /// No check is made on whether the decision level of the atom is tracked.
    pub unsafe fn level_unchecked(&self, atom: Atom) -> Option<LevelIndex> {
        *unsafe { self.atom_level_map.get_unchecked(atom as usize) }
    }

    /// Sets a given atom to have a given value, with a note of which decision this occurs after, if some decision has been made.
    ///
    /// # Safety
    /// No check is made on whether the atom is part of the valuation.
    pub unsafe fn set_value_unchecked(
        &mut self,
        literal: impl Borrow<CLiteral>,
        level: LevelIndex,
    ) -> AtomValue {
        let literal = literal.borrow();
        let atom = literal.atom();
        let value = literal.polarity();

        match self.value_of(atom) {
            None => unsafe {
                *self.valuation.get_unchecked_mut(atom as usize) = Some(value);
                *self.atom_level_map.get_unchecked_mut(atom as usize) = Some(level);
                AtomValue::NotSet
            },
            Some(v) if v == value => AtomValue::Same,

            Some(_) => AtomValue::Different,
        }
    }

    /// Clears the value of an atom, and adds the atom to the activity heap.
    ///
    /// # Safety
    /// No check is made on whether the atom is part of the valuation.
    pub unsafe fn drop_value(&mut self, atom: Atom) {
        unsafe {
            log::trace!(target: targets::VALUATION, "Cleared atom: {atom}");
            self.clear_value(atom);
            self.activity_heap.activate(atom as usize);
        }
    }
}

impl AtomDB {
    // /// The assignments made at `level`, in order of assignment.
    // ///
    // /// # Safety
    // /// No check is made to ensure the relevant number of assignments have been made.
    // pub unsafe fn assignments_at_unchecked(&self, level: LevelIndex) -> &[Assignment] {
    //     let level_start = *unsafe { self.level_indicies.get_unchecked(level as usize) };

    //     let level_end = if ((level + 1) as usize) < self.level_indicies.len() {
    //         *unsafe { self.level_indicies.get_unchecked((level + 1) as usize) }
    //     } else {
    //         self.assignments.len()
    //     };

    //     &self.assignments[level_start..level_end]
    // }

    // /// The assignments made at `level`, in order of assignment.
    // pub fn assignments_above(&self, level: LevelIndex) -> &[Assignment] {
    //     if let Some(&level_start) = self.level_indicies.get(level as usize) {
    //         &self.assignments[level_start..]
    //     } else {
    //         &[]
    //     }
    // }

    /// Removes levels above the given level index, if they exist.
    ///
    /// # Soundness
    /// Does not clear the *valuation* of the decision.
    pub fn clear_assigments_above(&mut self, level: LevelIndex) -> Vec<CLiteral> {
        // level_indicies stores with zero-indexing.
        // So, for example, the first assignment is accessed by assignments[level_indicies[0]].
        // This means, in particular, that all assignments made after level i can be cleared by clearing any assignment at and after assignments[level_indicies[0]].
        // And, as a corollary, that this method can not be used to clear any assignments at level zero.

        if let Some(&level_start) = self.trail.level_indicies.get(level as usize) {
            self.trail.level_indicies.split_off(level as usize);
            let assignments = self.trail.literals.split_off(level_start);
            for literal in &assignments {
                unsafe { self.drop_value(literal.atom()) }
            }
            assignments
        } else {
            Vec::default()
        }
    }

    // TODO: Requires check on if assumptions have been made
    // pub fn assumptions_made(&self) -> &[CLiteral] {
    //     &self.trail[0..self.level_indicies[self.initial_decision_level as usize - 1]]
    // }
}

impl Trail {
    /// Stores a consequence of the top decision level.
    pub fn store_assignment(&mut self, literal: CLiteral) {
        self.literals.push(literal);
    }

    /// True if some assumption has been made, false otherwise.
    pub fn assumption_is_made(&self) -> bool {
        self.initial_decision_level > 0
    }

    /// Returns the lowest decision level.
    /// Zero, if no assumptions has been made, otherwise some higher level.
    pub fn lowest_decision_level(&self) -> LevelIndex {
        self.initial_decision_level
    }

    /// The assignments made at the (current) top level, in order of assignment.
    pub fn top_level_assignments(&self) -> &[CLiteral] {
        if let Some(&level_start) = self.level_indicies.last() {
            &self.literals[level_start..]
        } else {
            &[]
        }
    }

    /// A count of how many decisions have been made.
    /// That is, the count of only those levels containing decisions (as opposed to assumptions).
    ///
    /// In other words, a count of how many decisions have been made.
    pub fn decision_count(&self) -> LevelIndex {
        (self.level_indicies.len() as LevelIndex) - self.initial_decision_level
    }

    /// Returns true if some decision is active, false otherwise (regardless of whether an assumption has been made).
    pub fn decision_is_made(&self) -> bool {
        self.decision_count() > 0
    }

    /// The current level.
    pub fn level(&self) -> LevelIndex {
        self.level_indicies.len() as LevelIndex
    }

    /// Removes the top level, if it exists.
    ///
    /// # Soundness
    /// Does not clear the *valuation* of the decision.
    pub fn forget_top_level(&mut self) -> Vec<CLiteral> {
        if let Some(top_start) = self.level_indicies.pop() {
            self.literals.split_off(top_start)
        } else {
            Vec::default()
        }
    }

    /// Takes the current list of assignments, leaving the default assignment container, until the list is restored.
    /// To be used in conjunction with [AtomDB::restore_assignments].
    pub fn take_assignments(&mut self) -> Vec<CLiteral> {
        std::mem::take(&mut self.literals)
    }

    /// Sets the current lists of assignments to `assignments`.
    /// To be used in conjunction with [AtomDB::take_assignments].
    pub fn restore_assignments(&mut self, assignents: Vec<CLiteral>) {
        self.literals = assignents;
    }
}
