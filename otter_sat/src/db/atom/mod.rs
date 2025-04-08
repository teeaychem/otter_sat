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

/// The atom database.
pub struct AtomDB {
    /// The previous (often partial) [valuation](Valuation) (or some randomised valuation).
    pub previous_valuation: Vec<bool>,
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
    pub fn new() -> Self {
        AtomDB {
            previous_valuation: Vec::default(),
        }
    }
}
