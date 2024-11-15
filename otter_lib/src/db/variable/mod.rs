mod activity;
mod valuation;
pub mod watch_db;

use crossbeam::channel::Sender;

use crate::{
    config::Activity,
    db::keys::ChoiceIndex,
    db::variable::watch_db::WatchDB,
    dispatch::{
        delta::{self},
        Dispatch,
    },
    generic::heap::IndexHeap,
    misc::log::targets::{self},
    structures::variable::Variable,
    types::gen::{self},
};

pub struct VariableDB {
    score_increment: Activity,

    watch_dbs: Vec<WatchDB>,

    internal_map: std::collections::HashMap<String, Variable>,
    external_map: Vec<String>,

    activity_heap: IndexHeap<Activity>,

    valuation: Vec<Option<bool>>,
    previous_valuation: Vec<bool>,
    choice_indicies: Vec<Option<ChoiceIndex>>,

    tx: Sender<Dispatch>,
}

impl VariableDB {
    pub fn new(tx: Sender<Dispatch>) -> Self {
        VariableDB {
            external_map: Vec::<String>::default(),
            internal_map: std::collections::HashMap::default(),

            watch_dbs: Vec::default(),

            score_increment: 1.0,
            activity_heap: IndexHeap::default(),

            valuation: Vec::default(),
            previous_valuation: Vec::default(),
            choice_indicies: Vec::default(),

            tx,
        }
    }

    // TODO: Maybe something more robust to internal revision
    pub fn count(&self) -> usize {
        self.valuation.len()
    }

    pub fn valuation(&self) -> &[Option<bool>] {
        &self.valuation
    }
}

impl VariableDB {
    pub fn internal_representation(&self, name: &str) -> Option<Variable> {
        self.internal_map.get(name).copied()
    }

    pub fn external_representation(&self, index: Variable) -> &String {
        &self.external_map[index as usize]
    }

    pub fn fresh_variable(&mut self, name: &str, previous_value: bool) -> Variable {
        let id = self.watch_dbs.len() as Variable;

        self.internal_map.insert(name.to_string(), id);
        self.external_map.push(name.to_string());

        self.activity_heap.add(id as usize, Activity::default());

        self.watch_dbs.push(WatchDB::new());
        self.valuation.push(None);
        self.previous_valuation.push(previous_value);
        self.choice_indicies.push(None);

        let delta = delta::Variable::Internalised(name.to_string(), id);
        self.tx.send(Dispatch::VariableDB(delta));

        id
    }
}

impl VariableDB {
    pub fn choice_index_of(&self, v_idx: Variable) -> Option<ChoiceIndex> {
        unsafe { *self.choice_indicies.get_unchecked(v_idx as usize) }
    }

    pub fn set_value(
        &mut self,
        v_idx: Variable,
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

    pub fn drop_value(&mut self, index: Variable) {
        log::trace!(target: targets::VALUATION, "Cleared: {index}");
        self.clear_value(index);
        self.activity_heap.activate(index as usize)
    }
}
