use crate::{
    config::{
        defaults::{self},
        ActivityType, Config,
    },
    context::{
        level::Level,
        store::{ClauseKey, ClauseStore},
    },
    generic::heap::FixedHeap,
    structures::{
        clause::{
            stored::{StoredClause, Watch},
            Clause,
        },
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
    score_increment: ActivityType,
    variables: Vec<Variable>,
    consequence_q: VecDeque<Literal>,
    string_map: HashMap<String, VariableId>,
    activity_heap: FixedHeap<ActivityType>,
}

impl VariableStore {
    pub fn id_of(&self, name: &str) -> Option<VariableId> {
        self.string_map.get(name).copied()
    }

    pub fn score_increment(&self) -> ActivityType {
        self.score_increment
    }

    pub fn activity_of(&self, index: usize) -> ActivityType {
        self.activity_heap.value_at(index)
    }

    pub fn activity_max(&self) -> Option<ActivityType> {
        self.activity_heap.peek_max_value()
    }

    pub fn rescore_activity(&mut self) {
        let heap_max = match self.activity_max() {
            Some(v) => v,
            None => ActivityType::MIN,
        };
        let rescale = ActivityType::max(heap_max, self.score_increment());

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

enum WatchCheck {
    NotFound,
    Updated,
    Witness,
    Check(Watch),
}

enum PropagationResult {
    Witness,
    Conflict,
    Success,
}

#[allow(clippy::collapsible_if)]
impl VariableStore {
    #[inline(always)]
    fn check_watch(&self, v_id: VariableId, watch: Watch, clause: &mut StoredClause) -> WatchCheck {
        if v_id != clause.get_watch(watch).v_id() {
            return WatchCheck::NotFound;
        }

        match clause.update_watch(watch, &self.variables) {
            Ok(_) => WatchCheck::Updated,
            Err(_) => match self.variables.polarity_of(clause.get_watch(watch).index()) {
                None => panic!("Watch has no value"),
                Some(value) => match value == clause.get_watch(watch).polarity() {
                    true => WatchCheck::Witness,
                    false => WatchCheck::Check(watch.switch()),
                },
            },
        }
    }

    #[inline(always)]
    fn propagate_watch(
        &self,
        literal: Literal,
        clause: &mut StoredClause,
        level: &mut Level,
    ) -> PropagationResult {
        match self.polarity_of(literal.index()) {
            None => match self.set_value(literal, level, Source::Clause(clause.key())) {
                Ok(_) => PropagationResult::Success,
                Err(e) => panic!("could not set watch {e:?}"),
            },
            Some(value) if literal.polarity() != value => PropagationResult::Conflict,
            Some(_) => PropagationResult::Witness,
        }
    }

    pub fn propagate(
        &mut self,
        literal: Literal,
        level: &mut Level,
        clauses: &mut ClauseStore,
    ) -> Result<(), ClauseKey> {
        // let not_watch_witness = |literal: Literal| {
        //     let the_variable = unsafe { self.variables.get_unchecked(literal.index()) };
        //     match the_variable.value() {
        //         None => true,
        //         Some(found_polarity) => found_polarity != literal.polarity(),
        //     }
        // };
        // println!("Propagating {} which has value {:?}", literal.index(), self.variables.polarity_of(literal.index()));

        let the_variable = self.variables.get_unsafe(literal.index());
        let the_variable_id = the_variable.id();
        let list_polarity = !literal.polarity();

        let mut index = 0;
        let mut length = the_variable.occurrence_length(list_polarity);

        'propagation_loop: while index < length {
            let clause_key = the_variable.occurrence_key_at_index(list_polarity, index);

            let clause = match clauses.retreive_carefully_mut(clause_key) {
                Some(stored_clause) => stored_clause,
                None => {
                    the_variable.remove_occurrence_at_index(list_polarity, index);
                    length -= 1;
                    continue 'propagation_loop;
                }
            };

            match self.check_watch(the_variable_id, Watch::A, clause) {
                WatchCheck::Witness => panic!("corrupted watch list"),
                WatchCheck::NotFound => {}
                WatchCheck::Updated => {
                    the_variable.remove_occurrence_at_index(list_polarity, index);
                    length -= 1;
                    continue 'propagation_loop;
                }
                WatchCheck::Check(unknown_watch) => {
                    let literal = clause.get_watch(unknown_watch);
                    match self.propagate_watch(literal, clause, level) {
                        PropagationResult::Conflict => return Err(clause_key),
                        PropagationResult::Witness => {}
                        PropagationResult::Success => {
                            self.consequence_q.push_back(literal);
                        }
                    }
                    index += 1;
                    continue 'propagation_loop;
                }
            };

            match self.check_watch(the_variable_id, Watch::B, clause) {
                WatchCheck::Witness => panic!("corrupted watch list"),
                WatchCheck::NotFound => {}
                WatchCheck::Updated => {
                    the_variable.remove_occurrence_at_index(list_polarity, index);
                    length -= 1;
                    continue 'propagation_loop;
                }
                WatchCheck::Check(unknown_watch) => {
                    let literal = clause.get_watch(unknown_watch);
                    match self.propagate_watch(literal, clause, level) {
                        PropagationResult::Conflict => return Err(clause_key),
                        PropagationResult::Witness => {}
                        PropagationResult::Success => {
                            self.consequence_q.push_back(literal);
                        }
                    }
                    index += 1;
                    continue 'propagation_loop;
                }
            }

            the_variable.remove_occurrence_at_index(list_polarity, index);
            length -= 1;
            continue 'propagation_loop;
        }
        Ok(())
    }

    pub fn tidy_queued_consequences(&mut self, stored_clauses: &mut ClauseStore) {
        while let Some(literal) = self.consequence_q.pop_front() {
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
                        if variable.id() == stored_clause.get_watch(Watch::A).v_id() {
                            match stored_clause.update_watch(Watch::A, &self.variables) {
                                Ok(_) => {
                                    variable.remove_occurrence_at_index(list_polarity, index);
                                    length -= 1;
                                }
                                Err(_) => {
                                    index += 1;
                                }
                            }
                        } else if variable.id() == stored_clause.get_watch(Watch::B).v_id() {
                            match stored_clause.update_watch(Watch::B, &self.variables) {
                                Ok(_) => {
                                    variable.remove_occurrence_at_index(list_polarity, index);
                                    length -= 1;
                                }
                                Err(_) => {
                                    index += 1;
                                }
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

    #[inline]
    #[allow(non_snake_case)]
    /// Bumps the activities of each variable in 'variables'
    /// If given a hint to the max activity the rescore check is performed once on the hint
    pub fn apply_VSIDS<V: Iterator<Item = usize>>(
        &mut self,
        variables: V,
        hint: Option<ActivityType>,
        config: &Config,
    ) {
        let activity = config.activity_conflict;
        match hint {
            Some(hint) => {
                if hint + activity > config.activity_max {
                    self.rescore_activity()
                }
                variables.for_each(|index| self.bump_activity(index));
            }
            None => {
                for index in variables {
                    if self.activity_of(index) + activity > config.activity_max {
                        self.rescore_activity()
                    }
                    self.bump_activity(index);
                }
            }
        }
        self.decay_activity(config);
    }

    pub fn clear_consequences(&mut self) {
        self.consequence_q.clear()
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
