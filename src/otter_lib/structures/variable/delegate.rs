use crate::{
    config::{
        defaults::{self},
        ActivityType, Config,
    },
    context::{
        level::{Level, LevelIndex},
        store::{ClauseKey, ClauseStore},
    },
    generic::heap::FixedHeap,
    structures::{
        literal::{Literal, LiteralSource},
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
    pub consequence_q: VecDeque<(Literal, LiteralSource, LevelIndex)>,
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

#[allow(clippy::collapsible_if)]
impl VariableStore {
    pub fn propagate(
        &mut self,
        literal: Literal,
        level: &mut Level,
        clauses: &mut ClauseStore,
    ) -> Result<(), ClauseKey> {
        let the_variable = self.variables.get_unsafe(literal.index());
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

            use crate::structures::clause::stored::WatchStatus;
            let update_result = clause.update_watch(literal, &self.variables);
            match update_result {
                Ok(WatchStatus::TwoWitness) | Ok(WatchStatus::TwoNone) => {
                    index += 1;
                    continue 'propagation_loop;
                }
                Ok(WatchStatus::Witness) | Ok(WatchStatus::None) => {
                    the_variable.remove_occurrence_at_index(list_polarity, index);
                    length -= 1;
                    continue 'propagation_loop;
                }
                Ok(_) => panic!("can't get conflict from update"),
                Err(()) => match self.polarity_of(unsafe { clause.get_unchecked(0) }.index()) {
                    Some(value) if unsafe { clause.get_unchecked(0) }.polarity() != value => {
                        return Err(clause_key);
                    }
                    None => {
                        push_back_consequence(
                            &mut self.consequence_q,
                            unsafe { *clause.get_unchecked(0) },
                            LiteralSource::Propagation(clause_key),
                            level.index(),
                        );
                    }
                    Some(_) => {}
                },
            }
            index += 1;
            continue 'propagation_loop;
        }
        Ok(())
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

    pub fn clear_consequences(&mut self, to: LevelIndex) {
        self.consequence_q.retain(|(_, _, c)| *c < to);
    }

    pub fn print_valuation(&self) {
        println!(
            "v {:?}",
            self.variables
                .slice()
                .iter()
                .enumerate()
                .filter_map(|(i, v)| match v.value() {
                    None => None,
                    Some(true) => Some(i as isize),
                    Some(false) => Some(-(i as isize)),
                })
                .collect::<Vec<_>>()
        );
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

pub fn push_back_consequence(
    consequence_q: &mut VecDeque<(Literal, LiteralSource, LevelIndex)>,
    literal: Literal,
    source: LiteralSource,
    level: LevelIndex,
) {
    // todo: improve
    // easy would be to keep track of pending of each variable, then direct lookup
    if !consequence_q.iter().any(|(l, _, _)| *l == literal) {
        consequence_q.push_back((literal, source, level))
    }
}
