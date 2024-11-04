use crate::{
    config::{
        defaults::{self},
        Config, VariableActivity,
    },
    context::{level::LevelIndex, stores::clause_key::ClauseKey, Context},
    generic::heap::IndexHeap,
    structures::{
        literal::{Literal, LiteralSource},
        variable::{list::VariableList, Variable, VariableId, WatchElement},
    },
};

use std::{
    collections::{HashMap, VecDeque},
    ops::{Deref, DerefMut},
};

pub enum ValueStatus {
    Set,
    NotSet,
    Match,
    Conflict,
}

pub struct VariableStore {
    external_map: Vec<String>,
    score_increment: VariableActivity,
    variables: Vec<Variable>,
    consequence_q: VecDeque<(Literal, LiteralSource, LevelIndex)>,
    string_map: HashMap<String, VariableId>,
    activity_heap: IndexHeap<VariableActivity>,
}

impl Default for VariableStore {
    fn default() -> Self {
        VariableStore {
            external_map: Vec::<String>::with_capacity(defaults::DEFAULT_VARIABLE_COUNT),
            score_increment: 1.0,
            variables: Vec::with_capacity(defaults::DEFAULT_VARIABLE_COUNT),
            consequence_q: VecDeque::with_capacity(defaults::DEFAULT_VARIABLE_COUNT),
            string_map: HashMap::with_capacity(defaults::DEFAULT_VARIABLE_COUNT),
            activity_heap: IndexHeap::new(defaults::DEFAULT_VARIABLE_COUNT),
        }
    }
}

impl Deref for VariableStore {
    type Target = [Variable];

    fn deref(&self) -> &Self::Target {
        &self.variables
    }
}

impl DerefMut for VariableStore {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.variables
    }
}

impl VariableStore {
    pub fn new(variables: Vec<Variable>) -> Self {
        VariableStore::with_capactiy(variables.len())
    }

    pub fn with_capactiy(variable_count: usize) -> Self {
        VariableStore {
            external_map: Vec::<String>::with_capacity(variable_count),
            score_increment: 1.0,
            variables: Vec::with_capacity(variable_count),
            consequence_q: VecDeque::with_capacity(variable_count),
            string_map: HashMap::with_capacity(variable_count),
            activity_heap: IndexHeap::new(variable_count),
        }
    }
}

impl VariableStore {
    pub fn id_of(&self, name: &str) -> Option<VariableId> {
        self.string_map.get(name).copied()
    }

    pub fn add_watch(&mut self, literal: Literal, element: WatchElement) {
        self.variables
            .get_unsafe(literal.index())
            .watch_added(element, literal.polarity());
    }

    pub fn remove_watch(&mut self, literal: Literal, key: ClauseKey) {
        self.variables
            .get_unsafe(literal.index())
            .watch_removed(key, literal.polarity());
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

    pub fn add_variable(&mut self, name: &str, variable: Variable) {
        self.string_map.insert(name.to_string(), variable.id());
        self.variables.push(variable);
        self.external_map.push(name.to_string());
        // self.consequence_buffer;
    }

    pub fn get_consequence(&mut self) -> Option<(Literal, LiteralSource, LevelIndex)> {
        self.consequence_q.pop_front()
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

    pub fn clear_consequences(&mut self, to: LevelIndex) {
        self.consequence_q.retain(|(_, _, c)| *c < to);
    }
}

impl VariableStore {
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

impl Context {
    pub fn q_literal(&mut self, lit: Literal, source: LiteralSource) -> Result<(), ValueStatus> {
        self.variables
            .set_value(lit, self.levels.top_mut(), source)?;

        // TODO: improve push back consequence
        self.variables
            .consequence_q
            .push_back((lit, source, self.levels.index()));

        Ok(())
    }
}
