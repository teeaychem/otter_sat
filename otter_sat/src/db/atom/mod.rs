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
    dispatch::Dispatch,
    generic::index_heap::IndexHeap,
    misc::log::targets::{self},
    structures::{
        atom::Atom,
        valuation::{vValuation, Valuation},
    },
    types::err::AtomDBError,
};

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
    decision_indicies: Vec<Option<DecisionLevelIndex>>,

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
        let mut db = AtomDB {
            watch_dbs: Vec::default(),

            activity_heap: IndexHeap::default(),

            valuation: Vec::default(),
            previous_valuation: Vec::default(),
            decision_indicies: Vec::default(),

            dispatcher,
            config: config.atom_db.clone(),
        };
        // A fresh atom is created so long as the atom count is within Atom::MAX
        // So, this is safe, for any reasonable atom specification.
        let the_true = unsafe { db.fresh_atom(true).unwrap_unchecked() };
        unsafe { db.set_value(the_true, true, None) };
        db
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

    /// A fresh atom, and a corresponding update to all the relevant data structures to ensure *unsafe* functions from the perspective of the compiler which do not check for the presence of an atom are safe.
    pub fn fresh_atom(&mut self, previous_value: bool) -> Result<Atom, AtomDBError> {
        let Ok(atom) = self.valuation.len().try_into() else {
            return Err(AtomDBError::AtomsExhausted);
        };

        self.activity_heap.add(atom as usize, 1.0);

        self.watch_dbs.push(WatchDB::default());
        self.valuation.push(None);
        self.previous_valuation.push(previous_value);
        self.decision_indicies.push(None);

        // if let Some(dispatcher) = &self.dispatcher {
        //     let delta = delta::AtomDB::Internalised(the_atoms);
        //     dispatcher(Dispatch::Delta(delta::Delta::AtomDB(delta)));
        // }

        Ok(atom)
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
