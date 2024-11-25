mod activity;
mod valuation;
pub mod watch_db;

use crossbeam::channel::Sender;

use crate::{
    config::{dbs::VariableDBConfig, Activity, Config},
    db::{keys::ChoiceIndex, variable::watch_db::WatchDB},
    dispatch::{
        library::delta::{self},
        Dispatch,
    },
    generic::heap::IndexHeap,
    misc::log::targets::{self},
    structures::{
        valuation::{Valuation, ValuationV},
        variable::Variable,
    },
    types::gen::{self},
};

pub struct VariableDB {
    watch_dbs: Vec<WatchDB>,

    internal_map: std::collections::HashMap<String, Variable>,
    external_map: Vec<String>,

    activity_heap: IndexHeap<Activity>,

    valuation: ValuationV,
    previous_valuation: Vec<bool>,
    choice_indicies: Vec<Option<ChoiceIndex>>,

    tx: Option<Sender<Dispatch>>,
    config: VariableDBConfig,
}

impl VariableDB {
    pub fn new(config: &Config, tx: Option<Sender<Dispatch>>) -> Self {
        VariableDB {
            external_map: Vec::<String>::default(),
            internal_map: std::collections::HashMap::default(),

            watch_dbs: Vec::default(),

            activity_heap: IndexHeap::default(),

            valuation: Vec::default(),
            previous_valuation: Vec::default(),
            choice_indicies: Vec::default(),

            tx,
            config: config.variable_db.clone(),
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

impl VariableDB {
    pub fn variable_representation(&self, name: &str) -> Option<Variable> {
        self.internal_map.get(name).copied()
    }

    pub fn external_representation(&self, index: Variable) -> &String {
        &self.external_map[index as usize]
    }

    pub fn fresh_variable(&mut self, name: &str, previous_value: bool) -> Variable {
        let the_variable = self.watch_dbs.len() as Variable;

        self.internal_map.insert(name.to_string(), the_variable);
        self.external_map.push(name.to_string());

        self.activity_heap.add(the_variable as usize, 1.0);
        // self.activity_heap.activate(id as usize);

        self.watch_dbs.push(WatchDB::new());
        self.valuation.push(None);
        self.previous_valuation.push(previous_value);
        self.choice_indicies.push(None);

        if let Some(tx) = &self.tx {
            let delta_rep = delta::VariableDB::ExternalRepresentation(name.to_string());
            tx.send(Dispatch::Delta(delta::Delta::VariableDB(delta_rep)));
            let delta = delta::VariableDB::Internalised(the_variable);
            tx.send(Dispatch::Delta(delta::Delta::VariableDB(delta)));
        }

        the_variable
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
        self.activity_heap.activate(index as usize);
    }
}
