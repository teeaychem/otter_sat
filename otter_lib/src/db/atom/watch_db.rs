//! A structure to record which clauses are watching an atom.
//!
//! # Overview
//!
//! The clauses watching an atom are distinguished by type, with the relevant distinctions set in the [WatchTag] enum.
//!
//! At present two distinctions are made:
//!
//! 1. Between binary clauses and other clauses.
//!    - This is made as in a binary clause the watched literals are never updated, and so the *other* literal can be recorded to avoid a trip to the clause itself.
//! 2. Between the value being watched.
//!    - This is made as the primary use for watch lists is to identify when the value of an atom has been updated.
//!      In this case, the the purpose of a watch is to note that the literal in the clause is now false, and so either:
//!        - The watch must be updated.
//!        - The clause now asserts some literal.
//!        - The formula being solved cannot be satisfied on the current valuation.
//!
//! So, in total each atom has four associated watch lists in it's watch database.
//!
//! Note, a unit clause (a clause containing one literal) never watches any atoms.
//!
//! The [WatchDB] structure does not have any associated mutating methods.
//! Instead, mutation of a [WatchDB] is through methods beloning to the [AtomDB].
//! Those methods are included in this file in order to access private members of the [WatchDB].
//!
//! # Use
//!
//! Watch lists are inspected and used during [boolean constraint propagation](crate::procedures::bcp).
//!
//! # Safety
//! As the [AtomDB] methods do not perform a check for whether a [WatchDB] exists for a given atom, these are all marked unsafe.
//!
//! At present, this is the only use of *unsafe* with respect to [WatchDB]s.

use crate::{
    db::{atom::AtomDB, keys::ClauseKey},
    structures::{atom::Atom, clause::ClauseKind, literal::abLiteral},
    types::err::{self},
};

/// The watcher of an atom.
pub enum WatchTag {
    /// A binary clause together with the *other* literal in the clause.
    Binary(abLiteral, ClauseKey),
    /// A long clause.
    Clause(ClauseKey),
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
    positive_binary: Vec<WatchTag>,

    /// A watch from a long clause for a value of `true`.
    positive_long: Vec<WatchTag>,

    /// A watch from a binary clause for a value of `false`.
    negative_binary: Vec<WatchTag>,

    /// A watch from a long clause for a value of `false`.
    negative_long: Vec<WatchTag>,
}

impl Default for WatchDB {
    fn default() -> Self {
        Self {
            positive_binary: Vec::default(),
            positive_long: Vec::default(),

            negative_binary: Vec::default(),
            negative_long: Vec::default(),
        }
    }
}

impl WatchDB {
    /// Returns the binary watchers of the atom for the given value.
    fn occurrences_binary(&mut self, value: bool) -> &mut Vec<WatchTag> {
        match value {
            true => &mut self.positive_binary,
            false => &mut self.negative_binary,
        }
    }

    /// Returns the long watchers of the atom for the given value.
    fn occurrences_long(&mut self, value: bool) -> &mut Vec<WatchTag> {
        match value {
            true => &mut self.positive_long,
            false => &mut self.negative_long,
        }
    }
}

impl AtomDB {
    /// Notes the given atom is being watched for being valued with the given value by the given watcher.
    ///
    /// # Safety
    /// No check is made on whether a [WatchDB] exists for the atom.
    pub unsafe fn add_watch_unchecked(&mut self, atom: Atom, value: bool, watcher: WatchTag) {
        match watcher {
            WatchTag::Binary(_, _) => match value {
                true => self
                    .watch_dbs
                    .get_unchecked_mut(atom as usize)
                    .positive_binary
                    .push(watcher),
                false => self
                    .watch_dbs
                    .get_unchecked_mut(atom as usize)
                    .negative_binary
                    .push(watcher),
            },
            WatchTag::Clause(_) => match value {
                true => self
                    .watch_dbs
                    .get_unchecked_mut(atom as usize)
                    .positive_long
                    .push(watcher),
                false => self
                    .watch_dbs
                    .get_unchecked_mut(atom as usize)
                    .negative_long
                    .push(watcher),
            },
        }
    }

    /// Notes the given atom is *no longer* being watched for being valued with the given value by the given watcher.
    ///
    /// # Safety
    /// No check is made on whether a [WatchDB] exists for the atom.
    /*
    If there's a guarantee keys appear at most once, the swap remove on keys could break early.
    Note also, as this shuffles the list any heuristics on traversal order of watches is void.
     */
    pub unsafe fn remove_watch_unchecked(
        &mut self,
        atom: Atom,
        value: bool,
        key: &ClauseKey,
    ) -> Result<(), err::Watch> {
        match key {
            ClauseKey::Original(_) | ClauseKey::Addition(_, _) => {
                let list = match value {
                    true => {
                        &mut self
                            .watch_dbs
                            .get_unchecked_mut(atom as usize)
                            .positive_long
                    }
                    false => {
                        &mut self
                            .watch_dbs
                            .get_unchecked_mut(atom as usize)
                            .negative_long
                    }
                };
                let mut index = 0;
                let mut limit = list.len();
                while index < limit {
                    let WatchTag::Clause(list_key) = list.get_unchecked(index) else {
                        return Err(err::Watch::NotLongInLong);
                    };

                    if list_key == key {
                        list.swap_remove(index);
                        limit -= 1;
                    } else {
                        index += 1;
                    }
                }
                Ok(())
            }
            ClauseKey::Unit(_) | ClauseKey::Binary(_) => Err(err::Watch::NotLongInLong),
        }
    }

    /// Returns the relevant collection of watchers for a given atom, clause type, and value.
    ///
    /// ```rust, ignore
    /// let binary_list = &mut *atom_db.get_watch_list_unchecked(atom, ClauseKind::Binary, false);
    /// ```
    ///
    /// # Safety
    /// No check is made on whether a [WatchDB] exists for the atom.
    ///
    /// Further, a pointer is returned --- --- to help simplify [bcp](crate::procedures::bcp) --- and so care should be taken to avoid creating aliases.
    pub unsafe fn get_watch_list_unchecked(
        &mut self,
        atom: Atom,
        kind: ClauseKind,
        value: bool,
    ) -> *mut Vec<WatchTag> {
        match kind {
            ClauseKind::Empty => panic!("!"),
            ClauseKind::Unit => panic!("!"),
            ClauseKind::Binary => self
                .watch_dbs
                .get_unchecked_mut(atom as usize)
                .occurrences_binary(value),
            ClauseKind::Long => self
                .watch_dbs
                .get_unchecked_mut(atom as usize)
                .occurrences_long(value),
        }
    }
}
