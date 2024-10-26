use crate::{
    config::{ActivityConflict, Config},
    context::store::{ClauseKey, ClauseStore},
    structures::{
        clause::stored::Watch,
        level::Level,
        literal::{Literal, Source},
        variable::{list::VariableList, ActivityRep, Variable, VariableId},
    },
};

const DEFAULT_VARIABLE_COUNT: usize = 1024;

pub enum Status {
    NotSet,
    Match,
    Conflict,
}

use std::{
    collections::{BinaryHeap, HashMap, VecDeque},
    ops::{Deref, DerefMut},
};

pub struct VariableActivity {
    pub index: usize,
    pub activity: ActivityConflict,
}

impl Ord for VariableActivity {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.activity.total_cmp(&other.activity)
    }
}

impl PartialOrd for VariableActivity {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for VariableActivity {
    fn eq(&self, other: &Self) -> bool {
        self.index == other.index && self.activity == other.activity
    }
}

impl Eq for VariableActivity {}

pub struct VariableStore {
    variables: Vec<Variable>,
    consequence_q: VecDeque<Literal>,
    pub string_map: HashMap<String, VariableId>, // pub consequence_buffer: Vec<(Source, Literal)>,
    pub activity_heap: BinaryHeap<VariableActivity>,
}

impl VariableActivity {
    pub fn new(index: usize, activity: ActivityConflict) -> Self {
        VariableActivity { index, activity }
    }
}

impl VariableStore {
    pub fn new(variables: Vec<Variable>) -> Self {
        let count = variables.len();

        VariableStore {
            variables,
            consequence_q: VecDeque::with_capacity(count),
            string_map: HashMap::with_capacity(count), // consequence_buffer: Vec::with_capacity(count),
            activity_heap: BinaryHeap::new(),
        }
    }

    pub fn with_capactiy(variable_count: usize) -> Self {
        VariableStore {
            variables: Vec::with_capacity(variable_count),
            consequence_q: VecDeque::with_capacity(variable_count),
            string_map: HashMap::with_capacity(variable_count), // consequence_buffer: Vec::with_capacity(variable_count),
            activity_heap: BinaryHeap::with_capacity(variable_count / 2),
        }
    }

    pub fn add_variable(&mut self, variable: Variable) {
        self.string_map.insert(variable.name.clone(), variable.id);
        self.variables.push(variable);
        // self.consequence_buffer;
    }
}

impl Default for VariableStore {
    fn default() -> Self {
        VariableStore {
            variables: Vec::with_capacity(DEFAULT_VARIABLE_COUNT),
            consequence_q: VecDeque::with_capacity(DEFAULT_VARIABLE_COUNT),
            string_map: HashMap::with_capacity(DEFAULT_VARIABLE_COUNT), // consequence_buffer: Vec::with_capacity(DEFAULT_VARIABLE_COUNT),
            activity_heap: BinaryHeap::with_capacity(DEFAULT_VARIABLE_COUNT / 2),
        }
    }
}

impl VariableStore {
    pub fn multiply_activity(&self, activity: ActivityRep) {
        for variable in &self.variables {
            variable.multiply_activity(activity);
        }
    }

    pub fn propagate(
        &mut self,
        literal: Literal,
        level: &mut Level,
        clause_store: &mut ClauseStore,
        config: &Config,
    ) -> Result<(), ClauseKey> {
        let not_watch_witness = |literal: Literal| {
            let the_variable = self.variables.get_unsafe(literal.index());
            match the_variable.polarity() {
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
                        // self.consequence_buffer
                        //     .push((Source::Clause(stored_clause.node_index()), watch_b));
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
                        // self.consequence_buffer
                        //     .push((Source::Clause(stored_clause.node_index()), watch_a));
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
                let the_variable = self.variables.get_unsafe(literal.index());
                match the_variable.polarity() {
                    None => true,
                    Some(found_polarity) => found_polarity != literal.polarity(),
                }
            };

            let variable = self.variables.get_unsafe(literal.index());

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
