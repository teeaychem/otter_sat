//! A database of 'atom related' things, accessed via fields on an [AtomDB] struct.
//!
//! Things include:
//! - Watch lists for each atom in the form of [WatchDB] structs, indexed by atoms.
//! - A current (often partial) [valuation](Valuation) and the previous valuation (or some randomised valuation).
//! - An [IndexHeap] recording the activty of atoms, where any atom without a value is 'active' on the heap.
//! - A record of which decision an atom was valued on.
//! - Internal and external name maps, for reading and writing [Atom]s, [Literal](crate::structures::literal::Literal)s, etc.

#[doc(hidden)]
pub mod activity;
#[doc(hidden)]
pub mod valuation;
pub mod watch_db;

use std::rc::Rc;

use crate::{
    config::{dbs::AtomDBConfig, Activity, Config},
    db::{atom::watch_db::WatchDB, DecisionLevelIndex},
    dispatch::{
        library::delta::{self},
        Dispatch,
    },
    generic::index_heap::IndexHeap,
    misc::log::targets::{self},
    structures::{
        atom::Atom,
        valuation::{vValuation, Valuation},
    },
};

/// The atom database.
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
    decision_indicies: Vec<Option<DecisionLevelIndex>>,

    /// A map from the external representation of an atom as a string to its internal representation.
    internal_map: std::collections::HashMap<String, Atom>,
    /// A map from the internal representation of an atom to its external representation, where the internal representation of atoms correspond indicies of the vector.
    external_map: Vec<String>,

    /// An optional function to send dispatches with.
    dispatcher: Option<Rc<dyn Fn(Dispatch)>>,

    /// A local configuration, typically derived from the configuration of a context.
    config: AtomDBConfig,
}

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
    pub fn new(config: &Config, dispatcher: Option<Rc<dyn Fn(Dispatch)>>) -> Self {
        AtomDB {
            external_map: Vec::<String>::default(),
            internal_map: std::collections::HashMap::default(),

            watch_dbs: Vec::default(),

            activity_heap: IndexHeap::default(),

            valuation: Vec::default(),
            previous_valuation: Vec::default(),
            decision_indicies: Vec::default(),

            dispatcher,
            config: config.atom_db.clone(),
        }
    }

    /// A count of atoms in the [AtomDB].
    // TODO: Maybe something more robust to internal revision
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

    /// The internal representation of an atom.
    pub fn internal_representation(&self, name: &str) -> Option<Atom> {
        self.internal_map.get(name).copied()
    }

    /// The external representation of an atom.
    pub fn external_representation(&self, index: Atom) -> &String {
        &self.external_map[index as usize]
    }

    /// Returns the value of an atom given by its exteranl representation
    pub fn value_of_external(&self, atom: &str) -> Option<bool> {
        match self.internal_map.get(atom) {
            Some(atom) => unsafe { self.valuation.value_of_unchecked(*atom) },
            None => None,
        }
    }

    /// A fresh atom, and a corresponding update to all the relevant data structures to ensure *unsafe* functions from the perspective of the compiler which do not check for the presence of an atom are safe.
    pub fn fresh_atom(&mut self, string: &str, previous_value: bool) -> Atom {
        let the_atoms = self.watch_dbs.len() as Atom;

        self.internal_map.insert(string.to_string(), the_atoms);
        self.external_map.push(string.to_string());

        self.activity_heap.add(the_atoms as usize, 1.0);

        self.watch_dbs.push(WatchDB::default());
        self.valuation.push(None);
        self.previous_valuation.push(previous_value);
        self.decision_indicies.push(None);

        if let Some(dispatcher) = &self.dispatcher {
            let delta_rep = delta::AtomDB::ExternalRepresentation(string.to_string());
            dispatcher(Dispatch::Delta(delta::Delta::AtomDB(delta_rep)));
            let delta = delta::AtomDB::Internalised(the_atoms);
            dispatcher(Dispatch::Delta(delta::Delta::AtomDB(delta)));
        }

        the_atoms
    }

    /// Which decision an atom was valued on.
    ///
    /// # Safety
    /// No check is made on whether a [WatchDB] exists for the atom.
    pub unsafe fn decision_index_of(&self, atom: Atom) -> Option<DecisionLevelIndex> {
        *self.decision_indicies.get_unchecked(atom as usize)
    }

    /// Sets a given atom to have a given value, with a note of which decision this occurs after, if some decision has been made.
    ///
    /// # Safety
    /// No check is made on whether a [WatchDB] exists for the atom.
    pub unsafe fn set_value(
        &mut self,
        atom: Atom,
        value: bool,
        level: Option<DecisionLevelIndex>,
    ) -> Result<AtomValue, AtomValue> {
        match self.value_of(atom) {
            None => {
                *self.valuation.get_unchecked_mut(atom as usize) = Some(value);
                *self.decision_indicies.get_unchecked_mut(atom as usize) = level;
                Ok(AtomValue::NotSet)
            }
            Some(v) if v == value => Ok(AtomValue::Same),
            Some(_) => Err(AtomValue::Different),
        }
    }

    /// Clears the value of an atom, and adds the atom to the activity heap.
    ///
    /// # Safety
    /// No check is made on whether a [WatchDB] exists for the atom.
    pub unsafe fn drop_value(&mut self, atom: Atom) {
        log::trace!(target: targets::VALUATION, "Cleared: {atom}");
        self.clear_value(atom);
        self.activity_heap.activate(atom as usize);
    }
}
