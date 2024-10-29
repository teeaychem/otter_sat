use crate::{
    config::{
        defaults::{self},
        ActivityConflict, Config,
    },
    context::store::{ClauseKey, ClauseStore},
    generic::heap::FixedHeap,
    structures::{
        clause::stored::Watch,
        level::Level,
        literal::{Literal, Source},
        variable::{list::VariableList, Variable, VariableId},
    },
};

use std::{
    collections::{HashMap, VecDeque},
    ops::{Deref, DerefMut},
};

pub enum Status {
    NotSet,
    Match,
    Conflict,
}

pub struct VariableStore {
    external_map: Vec<String>,
    score_increment: ActivityConflict,
    variables: Vec<Variable>,
    consequence_q: VecDeque<Literal>,
    string_map: HashMap<String, VariableId>,
    activity_heap: FixedHeap<ActivityConflict>,
}

impl VariableStore {
    pub fn id_of(&self, name: &str) -> Option<VariableId> {
        self.string_map.get(name).copied()
    }

    pub fn score_increment(&self) -> ActivityConflict {
        self.score_increment
    }

    pub fn activity_of(&self, index: usize) -> ActivityConflict {
        self.activity_heap.value_at(index)
    }

    pub fn activity_max(&self) -> Option<ActivityConflict> {
        self.activity_heap.peek_max_value()
    }

    pub fn rescore_activity(&mut self) {
        let heap_max = match self.activity_max() {
            Some(v) => v,
            None => ActivityConflict::MIN,
        };
        let rescale = ActivityConflict::max(heap_max, self.score_increment());

        let factor = 1.0 / rescale;
        self.activity_heap.reduce_all_with(factor);
        self.score_increment *= factor;
        self.activity_heap.bobble();
    }

    pub fn bump_activity(&mut self, index: usize) {
        self.activity_heap
            .update_one(index, self.activity_of(index) + self.score_increment())
    }

    pub fn decay_activity(&mut self, config: &Config) {
        let decay = config.decay_factor * 1e-3;
        let factor = 1.0 / (1.0 - decay);
        self.score_increment *= factor
    }

    pub fn heap_pop_most_active(&mut self) -> Option<usize> {
        self.activity_heap.pop_max()
    }

    pub fn heap_push(&mut self, index: usize) {
        self.activity_heap.activate(index)
    }
}

impl VariableStore {
    pub fn new(variables: Vec<Variable>) -> Self {
        let count = variables.len();

        VariableStore {
            external_map: Vec::<String>::with_capacity(count),
            score_increment: 1.0,
            variables,
            consequence_q: VecDeque::with_capacity(count),
            string_map: HashMap::with_capacity(count),
            activity_heap: FixedHeap::new(count, defaults::DEFAULT_ACTIVITY),
        }
    }

    pub fn with_capactiy(variable_count: usize) -> Self {
        VariableStore {
            external_map: Vec::<String>::with_capacity(variable_count),
            score_increment: 1.0,
            variables: Vec::with_capacity(variable_count),
            consequence_q: VecDeque::with_capacity(variable_count),
            string_map: HashMap::with_capacity(variable_count),
            activity_heap: FixedHeap::new(variable_count, defaults::DEFAULT_ACTIVITY),
        }
    }

    pub fn add_variable(&mut self, name: &str, variable: Variable) {
        self.string_map.insert(name.to_string(), variable.id);
        self.variables.push(variable);
        self.external_map.push(name.to_string());
        // self.consequence_buffer;
    }
}

impl Default for VariableStore {
    fn default() -> Self {
        VariableStore {
            external_map: Vec::<String>::with_capacity(defaults::DEFAULT_VARIABLE_COUNT),
            score_increment: 1.0,
            variables: Vec::with_capacity(defaults::DEFAULT_VARIABLE_COUNT),
            consequence_q: VecDeque::with_capacity(defaults::DEFAULT_VARIABLE_COUNT),
            string_map: HashMap::with_capacity(defaults::DEFAULT_VARIABLE_COUNT),
            activity_heap: FixedHeap::new(
                defaults::DEFAULT_VARIABLE_COUNT,
                defaults::DEFAULT_ACTIVITY,
            ),
        }
    }
}

impl VariableStore {
    pub fn propagate(
        &mut self,
        literal: Literal,
        level: &mut Level,
        clause_store: &mut ClauseStore,
        config: &Config,
    ) -> Result<(), ClauseKey> {
        let not_watch_witness = |literal: Literal| {
            let the_variable = unsafe { self.variables.get_unchecked(literal.index()) };
            match the_variable.value() {
                None => true,
                Some(found_polarity) => found_polarity != literal.polarity(),
            }
        };

        let the_variable = self.variables.get_unsafe(literal.index());
        let list_polarity = !literal.polarity();

        let mut index = 0;
        let mut length = the_variable.occurrence_length(list_polarity);

        'propagation_loop: while index < length {
            let clause_key = the_variable.occurrence_key_at_index(list_polarity, index);
            let maybe_stored_clause = clause_store.retreive_carefully_mut(clause_key);

            if maybe_stored_clause.is_none() {
                the_variable.remove_occurrence_at_index(list_polarity, index);
                length -= 1;
                continue 'propagation_loop;
            }

            let stored_clause = maybe_stored_clause.unwrap();

            let watch_a = stored_clause.get_watch(Watch::A);
            let watch_b = stored_clause.get_watch(Watch::B);

            if the_variable.id() == watch_a.v_id() {
                if not_watch_witness(watch_b) {
                    stored_clause.update_watch(Watch::A, &self.variables);
                }
            } else if the_variable.id() == watch_b.v_id() {
                if not_watch_witness(watch_a) {
                    stored_clause.update_watch(Watch::B, &self.variables);
                }
            } else {
                the_variable.remove_occurrence_at_index(list_polarity, index);
                length -= 1;
                continue 'propagation_loop;
            }

            let watch_a = stored_clause.get_watch(Watch::A);
            let watch_b = stored_clause.get_watch(Watch::B);

            if watch_a.v_id() != literal.v_id() && watch_b.v_id() != literal.v_id() {
                the_variable.remove_occurrence_at_index(list_polarity, index);
                length -= 1;
            } else {
                index += 1;
                let a_value = self.polarity_of(watch_a.index());
                let b_value = self.polarity_of(watch_b.index());

                match (a_value, b_value) {
                    (None, None) => {}
                    (Some(a), None) if a == watch_a.polarity() => {}
                    (Some(_), None) => {
                        match self.set_value(
                            watch_b,
                            level,
                            Source::Clause(stored_clause.node_index()),
                        ) {
                            Ok(_) => {}
                            Err(e) => panic!("could not set watch {e:?}"),
                        };
                        self.consequence_q.push_back(watch_b);
                    }
                    (None, Some(b)) if b == watch_b.polarity() => {}
                    (None, Some(_)) => {
                        match self.set_value(
                            watch_a,
                            level,
                            Source::Clause(stored_clause.node_index()),
                        ) {
                            Ok(_) => {}
                            Err(e) => panic!("could not set watch {e:?}"),
                        };
                        self.consequence_q.push_back(watch_a);
                    }
                    (Some(a), Some(b)) if a == watch_a.polarity() || b == watch_b.polarity() => {}
                    (Some(_), Some(_)) => {
                        match config.tidy_watches {
                            true => {
                                self.consequence_q.push_back(literal);
                                self.clear_queued_consequences(clause_store);
                            }
                            false => self.consequence_q.clear(),
                        }

                        return Err(clause_key);
                    }
                }
            }
        }
        Ok(())
    }

    fn clear_queued_consequences(&mut self, stored_clauses: &mut ClauseStore) {
        while let Some(literal) = self.consequence_q.pop_front() {
            let not_watch_witness = |literal: Literal| {
                let the_variable = unsafe { self.variables.get_unchecked(literal.index()) };
                match the_variable.value() {
                    None => true,
                    Some(found_polarity) => found_polarity != literal.polarity(),
                }
            };

            let variable = unsafe { self.variables.get_unchecked(literal.index()) };

            // process whether any change to the watch literals is required
            let list_polarity = !literal.polarity();

            let mut index = 0;
            let mut length = variable.occurrence_length(list_polarity);

            while index < length {
                let working_key = variable.occurrence_key_at_index(list_polarity, index);
                let working_clause = stored_clauses.retreive_carefully_mut(working_key);
                match working_clause {
                    None => {
                        variable.remove_occurrence_at_index(list_polarity, index);
                        length -= 1;
                    }
                    Some(stored_clause) => {
                        let watched_a = stored_clause.get_watch(Watch::A);
                        let watched_b = stored_clause.get_watch(Watch::B);

                        if variable.id() == watched_a.v_id() {
                            if not_watch_witness(watched_b) {
                                stored_clause.update_watch(Watch::A, &self.variables);
                            }
                            index += 1;
                        } else if variable.id() == watched_b.v_id() {
                            if not_watch_witness(watched_a) {
                                stored_clause.update_watch(Watch::B, &self.variables);
                            }
                            index += 1;
                        } else {
                            variable.remove_occurrence_at_index(list_polarity, index);
                            length -= 1;
                        }
                    }
                }
            }
        }
    }

    pub fn get_consequence(&mut self) -> Option<Literal> {
        self.consequence_q.pop_front()
    }

    pub fn push_back_consequence(&mut self, literal: Literal) {
        self.consequence_q.push_back(literal)
    }

    pub fn external_name(&self, index: usize) -> &String {
        &self.external_map[index]
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
