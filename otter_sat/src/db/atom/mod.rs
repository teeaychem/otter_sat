/*!
A database of 'atom related' things, accessed via fields on an [AtomDB] struct.

Things include:
- Watch lists for each atom in the form of [WatchDB] structs, indexed by atoms.
- A current (often partial) [valuation](Valuation) and the previous valuation (or some randomised valuation).
- An [IndexHeap] recording the activty of atoms, where any atom without a value is 'active' on the heap.
- A record of which decision an atom was valued on.
- Internal and external name maps, for reading and writing [Atom]s, [Literal]s, etc.
*/

#[doc(hidden)]
pub mod activity;
#[doc(hidden)]
pub mod valuation;
pub mod watch_db;

use std::borrow::Borrow;

use watch_db::{BinaryWatch, LongWatch};

use crate::{
    config::{dbs::AtomDBConfig, Activity, Config},
    db::{atom::watch_db::WatchDB, LevelIndex},
    generic::index_heap::IndexHeap,
    misc::log::targets::{self},
    structures::{
        atom::{Atom, ATOM_MAX},
        literal::{CLiteral, Literal},
        valuation::{vValuation, Valuation},
    },
    types::err::{self, AtomDBError},
};

use super::ClauseKey;

/// The atom database.
#[allow(dead_code)]
pub struct AtomDB {
    /// Watch lists for each atom in the form of [WatchDB] structs, indexed by atoms in the `watch_dbs` field.
    watch_dbs: Vec<WatchDB>,

    /// A current (often partial) [valuation](Valuation).
    valuation: vValuation,

    /// The previous (often partial) [valuation](Valuation) (or some randomised valuation).
    previous_valuation: Vec<bool>,

    /// An [IndexHeap] recording the activty of atoms, where any atom without a value is 'active' on the heap.
    activity_heap: IndexHeap<Activity>,

    /// A record of which decision an atom was valued on.
    decision_indicies: Vec<Option<LevelIndex>>,

    /// A local configuration, typically derived from the configuration of a context.
    config: AtomDBConfig,
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
        let mut db = AtomDB {
            watch_dbs: Vec::default(),

            activity_heap: IndexHeap::default(),

            valuation: Vec::default(),
            previous_valuation: Vec::default(),
            decision_indicies: Vec::default(),

            config: config.atom_db.clone(),
        };
        // A fresh atom is created so long as the atom count is within ATOM_MAX
        // So, this is safe, for any reasonable Atom specification.
        let the_true = unsafe { db.fresh_atom(true).unwrap_unchecked() };
        unsafe { db.set_value(CLiteral::new(the_true, true), None) };
        db
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

    /// A fresh atom --- on Ok the atom is part of the language of the context.
    ///
    /// If used, all the relevant data structures are updated to support access via the atom, and the safety of each unchecked is guaranteed.
    pub fn fresh_atom(&mut self, previous_value: bool) -> Result<Atom, AtomDBError> {
        let atom = match self.valuation.len().try_into() {
            // Note, ATOM_MAX over Atom::Max as the former is limited by the representation of literals, if relevant.
            Ok(atom) if atom <= ATOM_MAX => atom,
            _ => {
                return Err(AtomDBError::AtomsExhausted);
            }
        };

        self.activity_heap.add(atom as usize, 1.0);

        self.watch_dbs.push(WatchDB::default());
        self.valuation.push(None);
        self.previous_valuation.push(previous_value);
        self.decision_indicies.push(None);

        Ok(atom)
    }

    /// Which decision an atom was valued on.
    ///
    /// # Safety
    /// No check is made on whether the decision level of the atom is tracked.
    pub unsafe fn level_unchecked(&self, atom: Atom) -> Option<LevelIndex> {
        *self.decision_indicies.get_unchecked(atom as usize)
    }

    /// Sets a given atom to have a given value, with a note of which decision this occurs after, if some decision has been made.
    ///
    /// # Safety
    /// No check is made on whether the atom is part of the valuation.
    pub unsafe fn set_value(
        &mut self,
        literal: impl Borrow<CLiteral>,
        level: Option<LevelIndex>,
    ) -> AtomValue {
        let literal = literal.borrow();
        let atom = literal.atom();
        let value = literal.polarity();

        match self.value_of(atom) {
            None => {
                *self.valuation.get_unchecked_mut(atom as usize) = Some(value);
                *self.decision_indicies.get_unchecked_mut(atom as usize) = level;
                AtomValue::NotSet
            }
            Some(v) if v == value => AtomValue::Same,

            Some(_) => AtomValue::Different,
        }
    }

    /// Clears the value of an atom, and adds the atom to the activity heap.
    ///
    /// # Safety
    /// No check is made on whether the atom is part of the valuation.
    pub unsafe fn drop_value(&mut self, atom: Atom) {
        log::trace!(target: targets::VALUATION, "Cleared atom: {atom}");
        self.clear_value(atom);
        self.activity_heap.activate(atom as usize);
    }

    /// Adds `atom` being valued `value` to the binary clause wrapped in `watch_tag`.
    ///
    /// # Safety
    /// No check is made on whether a [WatchDB] exists for the atom.
    pub unsafe fn watch_binary_unchecked(&mut self, literal: &CLiteral, watch: BinaryWatch) {
        let atom = self.watch_dbs.get_unchecked_mut(literal.atom() as usize);
        match literal.polarity() {
            true => atom.positive_binary.push(watch),
            false => atom.negative_binary.push(watch),
        }
    }

    /// Adds `atom` being valued `value` to the clause wrapped in `watch_tag`.
    ///
    /// The counterpart of [unwatch_long_unchecked](AtomDB::unwatch_long_unchecked).
    ///
    /// # Safety
    /// No check is made on whether a [WatchDB] exists for the atom.
    pub unsafe fn watch_long_unchecked(&mut self, literal: &CLiteral, watch: LongWatch) {
        let atom = self.watch_dbs.get_unchecked_mut(literal.atom() as usize);
        let list = match literal.polarity() {
            true => &mut atom.positive_long,
            false => &mut atom.negative_long,
        };

        list.push(watch);
    }

    /// Removes `atom` being valued `value` to the clause wrapped in `watch_tag`.
    ///
    /// The counterpart of [watch_long_unchecked](AtomDB::watch_long_unchecked).
    ///
    /// # Safety
    /// No check is made on whether a [WatchDB] exists for the atom.
    /*
    If there's a guarantee keys appear at most once, the swap remove on keys could break early.
    Note also, as this shuffles the list any heuristics on traversal order of watches is void.
     */
    pub unsafe fn unwatch_long_unchecked(
        &mut self,
        literal: CLiteral,
        key: &ClauseKey,
    ) -> Result<(), err::ClauseDBError> {
        let atom = self.watch_dbs.get_unchecked_mut(literal.atom() as usize);
        match key {
            ClauseKey::Original(_) | ClauseKey::Addition(_, _) => {
                let list = match literal.polarity() {
                    true => &mut atom.positive_long,
                    false => &mut atom.negative_long,
                };

                let mut index = 0;
                let mut limit = list.len();

                while index < limit {
                    let list_key = list.get_unchecked(index).key;

                    if &list_key == key {
                        list.swap_remove(index);
                        limit -= 1;
                    } else {
                        index += 1;
                    }
                }
                Ok(())
            }
            ClauseKey::OriginalUnit(_)
            | ClauseKey::AdditionUnit(_)
            | ClauseKey::OriginalBinary(_)
            | ClauseKey::AdditionBinary(_) => Err(err::ClauseDBError::CorruptList),
        }
    }

    /// Returns the collection of binary watched clauses for `atom` to be valued with `value`.
    ///
    /// A pointer returned to help simplify [BCP](crate::procedures::bcp), though as BCP does not mutate the list of binary clauses, the pointer is marked const.
    ///
    /// # Safety
    /// No check is made on whether a [WatchDB] exists for the atom.
    pub unsafe fn watchers_binary_unchecked(&self, literal: &CLiteral) -> *const Vec<BinaryWatch> {
        let atom = self.watch_dbs.get_unchecked(literal.atom() as usize);

        match !literal.polarity() {
            true => &atom.positive_binary,
            false => &atom.negative_binary,
        }
    }

    /// Returns the collection of long watched clauses for `atom` to be valued with `value`.
    ///
    /// A mutable pointer returned to help simplify [BCP](crate::procedures::bcp).
    /// Specifically, to allow for multiple mutable borrows.
    /// As, both the watch list and valuation may be mutated during BCP.
    ///
    /// # Safety
    /// No check is made on whether a [WatchDB] exists for the atom.
    pub unsafe fn watchers_long_unchecked(&mut self, literal: &CLiteral) -> *mut Vec<LongWatch> {
        let atom = self.watch_dbs.get_unchecked_mut(literal.atom() as usize);

        match !literal.polarity() {
            true => &mut atom.positive_long,
            false => &mut atom.negative_long,
        }
    }
}
