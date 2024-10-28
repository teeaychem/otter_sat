use crate::{
    config::{ActivityConflict, Config},
    context::store::{ClauseKey, ClauseStore},
    generic::heap::FixedHeap,
    structures::{
        clause::stored::Watch,
        level::Level,
        literal::{Literal, Source},
        variable::{list::VariableList, Variable, VariableId},
    },
};

use std::pin::Pin;
use std::{
    collections::{HashMap, VecDeque},
    ops::{Deref, DerefMut},
};

const DEFAULT_VARIABLE_COUNT: usize = 1024;

pub enum Status {
    NotSet,
    Match,
    Conflict,
}

const DEFAULT_ACTIVITY: ActivityConflict = 0.0;

pub struct VariableStore {
    pub score_increment: ActivityConflict,
    variables: Vec<Pin<Box<Variable>>>,
    consequence_q: VecDeque<Literal>,
    pub string_map: HashMap<String, VariableId>,
    pub activity_heap: FixedHeap<ActivityConflict>,
}

impl VariableStore {
    pub fn new(variables: Vec<Variable>) -> Self {
        let count = variables.len();
        let mut pinned = vec![];
        for v in variables {
            let boxed = Box::new(v);
            let pin = Box::into_pin(boxed);
            pinned.push(pin);
        }

        VariableStore {
            score_increment: 1.0,
            variables: pinned,
            consequence_q: VecDeque::with_capacity(count),
            string_map: HashMap::with_capacity(count),
            activity_heap: FixedHeap::new(count, DEFAULT_ACTIVITY),
        }
    }

    pub fn with_capactiy(variable_count: usize) -> Self {
        VariableStore {
            score_increment: 1.0,
            variables: Vec::with_capacity(variable_count),
            consequence_q: VecDeque::with_capacity(variable_count),
            string_map: HashMap::with_capacity(variable_count),
            activity_heap: FixedHeap::new(variable_count, DEFAULT_ACTIVITY),
        }
    }

    pub fn add_variable(&mut self, variable: Variable) {
        self.string_map.insert(variable.name.clone(), variable.id);
        self.variables.push(Box::pin(variable));
        // self.consequence_buffer;
    }
}

impl Default for VariableStore {
    fn default() -> Self {
        VariableStore {
            score_increment: 1.0,
            variables: Vec::with_capacity(DEFAULT_VARIABLE_COUNT),
            consequence_q: VecDeque::with_capacity(DEFAULT_VARIABLE_COUNT),
            string_map: HashMap::with_capacity(DEFAULT_VARIABLE_COUNT),
            activity_heap: FixedHeap::new(DEFAULT_VARIABLE_COUNT, DEFAULT_ACTIVITY),
        }
    }
}

impl VariableStore {
    pub fn index_to_ptr(&self, index: usize) -> *const Variable {
        unsafe { &**self.variables.get_unchecked(index) }
    }

    pub fn propagate(
        &mut self,
        literal: Literal,
        level: &mut Level,
        clause_store: &mut ClauseStore,
        config: &Config,
    ) -> Result<(), ClauseKey> {
        let not_watch_witness = |literal: Literal| {
            let the_variable = unsafe { self.variables.get_unchecked(literal.index()) };
            match the_variable.polarity() {
                None => true,
                Some(found_polarity) => found_polarity != literal.polarity(),
            }
        };

        let the_variable = unsafe { literal.ptr.as_ref().unwrap() };
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
                let the_variable = unsafe { self.variables.get_unchecked(literal.index()) };
                match the_variable.polarity() {
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
}

impl Deref for VariableStore {
    type Target = [Pin<Box<Variable>>];

    fn deref(&self) -> &Self::Target {
        &self.variables
    }
}

impl DerefMut for VariableStore {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.variables
    }
}
