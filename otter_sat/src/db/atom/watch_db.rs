/*!
A structure to record which clauses are watching an atom.

# Theory

A core part of a solve is [Boolean Constraint Propagation](crate::procedures::bcp) (BCP).
In short, BCP is the observation that some literal in a clause must be true due to all other literals in the clause being false.

For example, given the clause p ∨ -q ∨ r and a valuation v such that p is false and q is true, the clause is true on the valution *only if* r is (made) true --- in other contexts, given the valuation and clause specified it is said the clause 'asserts' r.

Note, BCP only applies when:
- There is exactly one literal without a value.
- All other literals conflict with the background valuation.

This motivates the use of two watches:
- One watch on a literal without a value, to note the clause is a candidate for BCP to be applied at some point.
- One watch on any other literal which does not conflict with the background valuation, if possible.
  + For, if it is *only* possible to watch some other literal which conflicts with the current valuation, the other literal must be true.

The watch database records which clauses are watching an atoms, and for the implementation of watching literals see the way clauses are stored in a database [dbClause](crate::db::clause::db_clause) and in particular the associated methods [initialise_watches](crate::db::clause::db_clause::dbClause::initialise_watches) and [update_watch](crate::db::clause::db_clause::dbClause::initialise_watches).

# Literature

[The art of computer programming, Volume 4](https://www-cs-faculty.stanford.edu/~knuth/taocp.html) discusses watched literals in the *Lazy data structures* section of *Backtracking Algorithms*.
And, Knuth attributes the introduction of watched literals to [An Empirical Comparison of Backtracking Algorithms](https://doi.org/10.1109/TPAMI.1982.4767250).

It seems general use of watched literals followed from [Chaff](https://dl.acm.org/doi/10.1145/378239.379017).[^patent]
[^patent]: [US7418369B2](https://patents.google.com/patent/US7418369B2/en) is a patent covering Chaff, though at the time of writing the status of the patent is 'Expired'.

The given implentation of watch literals follows [Optimal implementation of watched literals and more general techniques](https://www.jair.org/index.php/jair/article/view/10839).

# Implementation

The clauses watching an atom are distinguished by type.

At present two distinctions are made:

1. Between binary clauses and other clauses.
   - This is made as in a binary clause the watched literals are never updated, and so the *other* literal can be recorded to avoid a trip to the clause itself.
2. Between the value being watched.
   - This is made as the primary use for watch lists is to identify when the value of an atom has been updated.
     In this case, the the purpose of a watch is to note that the literal in the clause is now false, and so either:
       - The watch must be updated.
       - The clause now asserts some literal.
       - The formula being solved cannot be satisfied on the current valuation.

So, in total each atom has four associated watch lists in it's watch database.

Note, a unit clause (a clause containing one literal) never watches any atoms.

The [WatchDB] structure does not have any associated mutating methods.
Instead, mutation of a [WatchDB] is through methods beloning to the [AtomDB](crate::db::atom::AtomDB).
Those methods are included in this file in order to access private members of the [WatchDB].

# Use

Watch lists are inspected and used during [boolean constraint propagation](crate::procedures::bcp).

# Watches and witnesses

The list of long clauses watchers of an atom contains, at present, only the keys of the watching clauses.

In principle, this list could be enhanced, I believe a technique like the following is employed by MiniSAT:

- For each watch, include the *other* watched literal at the time the watch was made.
- BCP then examines the value of thie literal before requesting the update to a watched literal.
- And, if the literal witnesses satisfiability of the clause, no update to watches is made.

This is sound, as a satisfied clause can never be used for propagation.
And, so in particular, a backjump must be made before the witness was set in order for the clause to be of interest.

The technique can be implemented with minimal changes.
For example:

- Update the [LongWatch] structure is updated to contain a field for the literal.
- On creation of a long watch, add the literal at the watched indexed to the watch.
- During BCP, first check the value of the other literal.

In principle, this may save unnecessary access to the clause database (as there may be no need to examine the clause).
Though, at the cost of fragmenting access to the atom database (as a check of the other literal is separate from checks after accessing the clause).
And, in practice, it seems the cost of fragmentation is greater than that of unnecessary access.

# Safety
As the [AtomDB](crate::db::atom::AtomDB) methods do not perform a check for whether a [WatchDB] exists for a given atom, these are all marked unsafe.

At present, this is the only use of *unsafe* with respect to [WatchDB]s.
*/

use crate::{db::keys::ClauseKey, structures::literal::CLiteral};

/// A binary clause together with the *other* literal in the clause.
pub struct BinaryWatch {
    pub literal: CLiteral,
    pub key: ClauseKey,
}

impl BinaryWatch {
    pub fn new(literal: CLiteral, key: ClauseKey) -> Self {
        Self { literal, key }
    }
}

/// A long clause watch of an atom.
#[derive(PartialEq, Eq)]
pub struct LongWatch {
    pub key: ClauseKey,
}

impl LongWatch {
    pub fn new(key: ClauseKey) -> Self {
        LongWatch { key }
    }
}

/// The status of a watched literal, relative to some given valuation.
#[derive(Clone, Copy, PartialEq)]
pub enum WatchStatus {
    /// The polarity of the watched literal matches the valuation of the atom on the given valuation.\
    /// E.g. if the literal is -p, then p is valued 'false' on the given valuation.
    Witness,

    /// The watched literal has no value on the given valuation.
    None,

    /// The polarity of the watched literal does not match the valuation of the atom on the given valuation.\
    /// E.g. if the literal is -p and p has value 'true' on the given valuation.
    Conflict,
}

/// The watchers of an atom, distinguished by length of clause and which value of the atom is under watch.
pub struct WatchDB {
    /// A watch from a binary clause for a value of `true`.
    pub(super) positive_binary: Vec<BinaryWatch>,

    /// A watch from a binary clause for a value of `false`.
    pub(super) negative_binary: Vec<BinaryWatch>,

    /// A watch from a long clause for a value of `true`.
    pub(super) positive_long: Vec<LongWatch>,

    /// A watch from a long clause for a value of `false`.
    pub(super) negative_long: Vec<LongWatch>,
}

impl Default for WatchDB {
    fn default() -> Self {
        Self {
            positive_binary: Vec::default(),
            negative_binary: Vec::default(),

            positive_long: Vec::default(),
            negative_long: Vec::default(),
        }
    }
}
