use std::borrow::Borrow;

use crossbeam::channel::Sender;

use crate::{
    config::{Config, VariableActivity},
    db::keys::{ClauseKey, VariableIndex},
    dispatch::{
        delta::{self},
        Dispatch,
    },
    generic::heap::IndexHeap,
    structures::{
        literal::{Literal, LiteralT},
        valuation::Valuation,
        variable::Variable,
    },
    types::{
        clause::WatchElement,
        errs::{self},
    },
};

pub struct VariableDB {
    external_map: Vec<String>,
    score_increment: VariableActivity,
    variables: Vec<Variable>,
    string_map: std::collections::HashMap<String, VariableIndex>,
    activity_heap: IndexHeap<VariableActivity>,
    valuation: Vec<Option<bool>>,
    past_valuation: Vec<bool>,
    tx: Sender<Dispatch>,
}

impl std::ops::Deref for VariableDB {
    type Target = [Variable];

    fn deref(&self) -> &Self::Target {
        &self.variables
    }
}

impl std::ops::DerefMut for VariableDB {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.variables
    }
}

impl VariableDB {
    pub fn new(tx: Sender<Dispatch>) -> Self {
        VariableDB {
            external_map: Vec::<String>::default(),
            score_increment: 1.0,
            variables: Vec::default(),

            string_map: std::collections::HashMap::default(),
            activity_heap: IndexHeap::default(),

            valuation: Vec::default(),
            past_valuation: Vec::default(),

            tx,
        }
    }
}

impl VariableDB {
    pub fn id_of(&self, name: &str) -> Option<VariableIndex> {
        self.string_map.get(name).copied()
    }

    pub fn add_watch<L: Borrow<Literal>>(&mut self, literal: L, element: WatchElement) {
        self.variables
            .get_unsafe(literal.borrow().index())
            .watch_added(element, literal.borrow().polarity());
    }

    pub fn remove_watch<L: Borrow<Literal>>(
        &mut self,
        literal: L,
        key: ClauseKey,
    ) -> Result<(), errs::Watch> {
        self.variables
            .get_unsafe(literal.borrow().index())
            .watch_removed(key, literal.borrow().polarity())
    }

    pub fn heap_pop_most_active(&mut self) -> Option<usize> {
        self.activity_heap.pop_max()
    }

    pub fn retract_valuation(&mut self, index: usize) {
        log::trace!(target: crate::log::targets::VALUATION, "Cleared: {index}");
        unsafe {
            self.get_unchecked_mut(index).set_value(None, None);
        }
        self.activity_heap.activate(index)
    }

    pub fn fresh_variable(&mut self, name: &str, previous_value: bool) -> &Variable {
        let id = self.variables.len() as VariableIndex;
        let the_variable = Variable::new(id, previous_value);

        let delta = delta::Variable::Internalised(name.to_string(), the_variable.id());
        self.tx.send(Dispatch::VariableDB(delta));

        self.string_map.insert(name.to_string(), the_variable.id());
        self.external_map.push(name.to_string());

        self.activity_heap
            .insert(the_variable.index(), VariableActivity::default());

        self.variables.push(the_variable);
        self.valuation.push(None);
        self.past_valuation.push(previous_value);

        self.variables.last().expect("added lines above…")
    }

    pub fn external_name(&self, index: usize) -> &String {
        &self.external_map[index]
    }

    #[inline]
    #[allow(non_snake_case)]
    /// Bumps the activities of each variable in 'variables'
    /// If given a hint to the max activity the rescore check is performed once on the hint
    pub fn apply_VSIDS<V: Iterator<Item = usize>>(&mut self, variables: V, config: &Config) {
        for index in variables {
            if self.activity_of(index) + config.activity_conflict > config.activity_max {
                self.rescore_activity()
            }
            self.bump_activity(index);
        }

        self.exponent_activity(config);
    }
}

impl VariableDB {
    fn activity_of(&self, index: usize) -> VariableActivity {
        *self.activity_heap.value_at(index)
    }

    fn bump_activity(&mut self, index: usize) {
        self.activity_heap
            .update_one(index, self.activity_of(index) + self.score_increment)
    }

    fn exponent_activity(&mut self, config: &Config) {
        let decay = config.variable_decay * 1e-3;
        let factor = 1.0 / (1.0 - decay);
        self.score_increment *= factor
    }

    fn activity_max(&self) -> Option<VariableActivity> {
        self.activity_heap.peek_max_value().copied()
    }

    fn rescore_activity(&mut self) {
        let heap_max = self.activity_max().unwrap_or(VariableActivity::MIN);
        let rescale = VariableActivity::max(heap_max, self.score_increment);

        let factor = 1.0 / rescale;
        let rescale = |v: &VariableActivity| v * factor;
        self.activity_heap.apply_to_all(rescale);
        self.score_increment *= factor;
        self.activity_heap.reheap();
    }
}
