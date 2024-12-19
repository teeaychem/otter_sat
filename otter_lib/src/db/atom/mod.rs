mod activity;
mod valuation;
pub mod watch_db;

use std::rc::Rc;

use crate::{
    config::{dbs::AtomDBConfig, Activity, Config},
    db::{atom::watch_db::WatchDB, keys::ChoiceIndex},
    dispatch::{
        library::delta::{self},
        Dispatch,
    },
    generic::heap::IndexHeap,
    misc::log::targets::{self},
    structures::{
        atom::Atom,
        valuation::{vValuation, Valuation},
    },
    types::gen::{self},
};

pub struct AtomDB {
    watch_dbs: Vec<WatchDB>,

    internal_map: std::collections::HashMap<String, Atom>,
    external_map: Vec<String>,

    activity_heap: IndexHeap<Activity>,

    valuation: vValuation,
    previous_valuation: Vec<bool>,
    choice_indicies: Vec<Option<ChoiceIndex>>,

    dispatcher: Option<Rc<dyn Fn(Dispatch)>>,
    config: AtomDBConfig,
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
            choice_indicies: Vec::default(),

            dispatcher,
            config: config.atom_db.clone(),
        }
    }

    // TODO: Maybe something more robust to internal revision
    pub fn count(&self) -> usize {
        self.valuation.len()
    }

    pub fn valuation(&self) -> &impl Valuation {
        &self.valuation
    }
}

impl AtomDB {
    pub fn atom_representation(&self, name: &str) -> Option<Atom> {
        self.internal_map.get(name).copied()
    }

    pub fn external_representation(&self, index: Atom) -> &String {
        &self.external_map[index as usize]
    }

    pub fn fresh_atom(&mut self, name: &str, previous_value: bool) -> Atom {
        let the_atoms = self.watch_dbs.len() as Atom;

        self.internal_map.insert(name.to_string(), the_atoms);
        self.external_map.push(name.to_string());

        self.activity_heap.add(the_atoms as usize, 1.0);
        // self.activity_heap.activate(id as usize);

        self.watch_dbs.push(WatchDB::new());
        self.valuation.push(None);
        self.previous_valuation.push(previous_value);
        self.choice_indicies.push(None);

        if let Some(dispatcher) = &self.dispatcher {
            let delta_rep = delta::AtomDB::ExternalRepresentation(name.to_string());
            dispatcher(Dispatch::Delta(delta::Delta::AtomDB(delta_rep)));
            let delta = delta::AtomDB::Internalised(the_atoms);
            dispatcher(Dispatch::Delta(delta::Delta::AtomDB(delta)));
        }

        the_atoms
    }
}

impl AtomDB {
    pub fn choice_index_of(&self, v_idx: Atom) -> Option<ChoiceIndex> {
        unsafe { *self.choice_indicies.get_unchecked(v_idx as usize) }
    }

    pub fn set_value(
        &mut self,
        v_idx: Atom,
        polarity: bool,
        level: Option<ChoiceIndex>,
    ) -> Result<gen::Value, gen::Value> {
        match self.value_of(v_idx) {
            None => unsafe {
                *self.valuation.get_unchecked_mut(v_idx as usize) = Some(polarity);
                *self.choice_indicies.get_unchecked_mut(v_idx as usize) = level;
                Ok(gen::Value::NotSet)
            },
            Some(v) if v == polarity => Ok(gen::Value::Match),
            Some(_) => Err(gen::Value::Conflict),
        }
    }

    pub fn drop_value(&mut self, index: Atom) {
        log::trace!(target: targets::VALUATION, "Cleared: {index}");
        self.clear_value(index);
        self.activity_heap.activate(index as usize);
    }
}
