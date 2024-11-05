use crate::{
    config::{self, ClauseActivity, Config},
    context::stores::{
        activity_glue::ActivityGlue, variable::VariableStore, ClauseKey, FormulaIndex,
    },
    generic::heap::IndexHeap,
    structures::{
        clause::{
            stored::{ClauseSource, StoredClause},
            Clause,
        },
        literal::Literal,
        variable::{WatchElement, WatchError},
    },
};

#[derive(Debug, Clone, Copy)]
pub enum ClauseStoreError {
    TransferBinary,
    TransferWatch,
    MissingLearned,
    InvalidKeyToken,
    InvalidKeyIndex,
    EmptyClause,
    UnitClause,
    StorageExhausted,
}

impl From<WatchError> for ClauseStoreError {
    fn from(_: WatchError) -> Self {
        ClauseStoreError::TransferWatch
    }
}

pub struct ClauseStore {
    counts: ClauseStoreCounts,
    keys: Vec<ClauseKey>,
    formula: Vec<StoredClause>,

    pub binary_graph: Vec<Vec<ClauseKey>>,
    learned: Vec<Option<StoredClause>>,

    pub learned_slots: FormulaIndex,

    pub resolution_graph: Vec<Vec<Vec<ClauseKey>>>,
    binary: Vec<StoredClause>,

    pub learned_activity: IndexHeap<ActivityGlue>,
    pub learned_increment: ClauseActivity,
}

pub struct ClauseStoreCounts {
    formula: FormulaIndex,
    binary: FormulaIndex,
    learned: FormulaIndex,
}

#[allow(clippy::derivable_impls)]
impl Default for ClauseStoreCounts {
    fn default() -> Self {
        ClauseStoreCounts {
            formula: 0,
            binary: 0,
            learned: 0,
        }
    }
}

#[allow(clippy::derivable_impls)]
impl Default for ClauseStore {
    fn default() -> Self {
        ClauseStore {
            counts: ClauseStoreCounts::default(),
            keys: Vec::new(),
            formula: Vec::new(),
            learned: Vec::new(),
            learned_slots: 0,
            binary: Vec::new(),
            binary_graph: Vec::new(),
            resolution_graph: Vec::new(),
            learned_activity: IndexHeap::default(),
            learned_increment: ClauseActivity::default(),
        }
    }
}

impl ClauseStore {
    pub fn with_capacity(capacity: usize) -> Self {
        ClauseStore {
            counts: ClauseStoreCounts::default(),
            keys: Vec::new(),
            formula: Vec::with_capacity(capacity),
            learned: Vec::with_capacity(capacity),
            learned_slots: 0,
            binary: Vec::new(),
            binary_graph: Vec::with_capacity(capacity),
            resolution_graph: Vec::with_capacity(capacity),
            learned_activity: IndexHeap::new(capacity),
            learned_increment: ClauseActivity::default(),
        }
    }
}

impl ClauseStore {
    fn new_formula_id(&mut self) -> Result<ClauseKey, ClauseStoreError> {
        if self.counts.formula == FormulaIndex::MAX {
            return Err(ClauseStoreError::StorageExhausted);
        }
        let key = ClauseKey::Formula(self.counts.formula);
        self.counts.formula += 1;
        Ok(key)
    }

    fn new_binary_id(&mut self) -> Result<ClauseKey, ClauseStoreError> {
        if self.counts.binary == FormulaIndex::MAX {
            return Err(ClauseStoreError::StorageExhausted);
        }
        let key = ClauseKey::Binary(self.counts.binary);
        self.counts.binary += 1;
        Ok(key)
    }

    fn new_learned_id(&mut self) -> Result<ClauseKey, ClauseStoreError> {
        if self.learned_slots == FormulaIndex::MAX {
            return Err(ClauseStoreError::StorageExhausted);
        }
        let key = ClauseKey::Learned(self.learned_slots, 0);
        self.learned_slots += 1;
        Ok(key)
    }
}

impl ClauseStore {
    pub fn get_carefully(&self, key: ClauseKey) -> Option<&StoredClause> {
        match key {
            ClauseKey::Formula(index) => self.formula.get(index as usize),
            ClauseKey::Binary(index) => self.binary.get(index as usize),
            ClauseKey::Learned(index, token) => match self.learned.get(index as usize) {
                Some(Some(clause)) => match clause.key() {
                    ClauseKey::Learned(_, clause_token) if clause_token == token => Some(clause),
                    _ => None,
                },
                _ => None,
            },
        }
    }

    pub fn get(&self, key: ClauseKey) -> Result<&StoredClause, ClauseStoreError> {
        match key {
            ClauseKey::Formula(index) => unsafe { Ok(self.formula.get_unchecked(index as usize)) },
            ClauseKey::Binary(index) => unsafe { Ok(self.binary.get_unchecked(index as usize)) },
            ClauseKey::Learned(index, token) => unsafe {
                match self.learned.get_unchecked(index as usize) {
                    Some(clause) => match clause.key() {
                        ClauseKey::Learned(_, clause_token) if clause_token == token => Ok(clause),
                        _ => Err(ClauseStoreError::InvalidKeyToken),
                    },
                    None => Err(ClauseStoreError::InvalidKeyIndex),
                }
            },
        }
    }

    pub fn get_carefully_mut(&mut self, key: ClauseKey) -> Option<&mut StoredClause> {
        match key {
            ClauseKey::Formula(index) => self.formula.get_mut(index as usize),
            ClauseKey::Binary(index) => self.binary.get_mut(index as usize),
            ClauseKey::Learned(index, token) => match self.learned.get_mut(index as usize) {
                Some(Some(clause)) => match clause.key() {
                    ClauseKey::Learned(_, clause_token) if clause_token == token => Some(clause),
                    _ => None,
                },
                _ => None,
            },
        }
    }

    pub fn get_mut(&mut self, key: ClauseKey) -> Result<&mut StoredClause, ClauseStoreError> {
        match key {
            ClauseKey::Formula(index) => unsafe {
                Ok(self.formula.get_unchecked_mut(index as usize))
            },
            ClauseKey::Binary(index) => unsafe {
                Ok(self.binary.get_unchecked_mut(index as usize))
            },
            ClauseKey::Learned(index, token) => unsafe {
                match self.learned.get_unchecked_mut(index as usize) {
                    Some(clause) => match clause.key() {
                        ClauseKey::Learned(_, clause_token) if clause_token == token => Ok(clause),
                        _ => Err(ClauseStoreError::InvalidKeyToken),
                    },
                    None => Err(ClauseStoreError::InvalidKeyIndex),
                }
            },
        }
    }

    pub fn insert(
        &mut self,
        source: ClauseSource,
        clause: Vec<Literal>,
        subsumed: Vec<Literal>,
        variables: &mut VariableStore,
        resolution_keys: Option<Vec<ClauseKey>>,
    ) -> Result<ClauseKey, ClauseStoreError> {
        match clause.len() {
            0 => Err(ClauseStoreError::EmptyClause),
            1 => Err(ClauseStoreError::UnitClause),
            2 => {
                let key = self.new_binary_id()?;
                self.binary.push(StoredClause::new_from(
                    key, clause, subsumed, source, variables,
                ));
                self.binary_graph.push(resolution_keys.unwrap_or_default());
                Ok(key)
            }
            _ => match source {
                ClauseSource::Formula => {
                    let key = self.new_formula_id()?;
                    self.formula.push(StoredClause::new_from(
                        key, clause, subsumed, source, variables,
                    ));
                    Ok(key)
                }
                ClauseSource::Resolution => {
                    log::trace!(target: crate::log::targets::CLAUSE_STORE, "Learning clause {}", clause.as_string());
                    self.counts.learned += 1;
                    match self.keys.len() {
                        0 => {
                            let key = self.new_learned_id()?;
                            let the_clause =
                                StoredClause::new_from(key, clause, subsumed, source, variables);

                            let value = ActivityGlue {
                                activity: ClauseActivity::default(),
                                lbd: the_clause.lbd(variables),
                            };

                            self.learned.push(Some(the_clause));

                            self.learned_activity.insert(key.index(), value);
                            self.resolution_graph.push(vec![
                                resolution_keys.expect("missing resolution info for learnt")
                            ]);

                            // assert_eq!(self.resolution_graph[key.index()].len(), 1);
                            Ok(key)
                        }
                        _ => unsafe {
                            let key = self.keys.pop().unwrap().retoken()?;
                            let the_clause =
                                StoredClause::new_from(key, clause, subsumed, source, variables);

                            let value = ActivityGlue {
                                activity: ClauseActivity::default(),
                                lbd: the_clause.lbd(variables),
                            };

                            *self.learned.get_unchecked_mut(key.index()) = Some(the_clause);
                            self.learned_activity.insert(key.index(), value);
                            self.resolution_graph[key.index()]
                                .push(resolution_keys.expect("missing resolution info for learnt"));
                            Ok(key)
                        },
                    }
                }
            },
        }
    }

    pub fn clause_count(&self) -> usize {
        (self.counts.formula + self.counts.learned + self.counts.binary) as usize
    }

    pub fn formula_clauses(&self) -> impl Iterator<Item = impl Iterator<Item = Literal> + '_> + '_ {
        self.formula
            .iter()
            .map(|clause| clause.literal_slice().iter().copied())
            .chain(
                self.binary
                    .iter()
                    .map(|clause| clause.literal_slice().iter().copied()),
            )
    }

    /*
    Cases:
    - Formula
      A new binary clause is created, and all occurrences of the formula clause in watch lists are removed

    - Learned
      Removes a long clause and then:
      - Replaces the key with a new binary key
      - Updates notifies literals of the new watch
      - Adds the clause to the binary vec
        This order is mostly dictated by the borrow checker
    */
    pub fn transfer_to_binary(
        &mut self,
        key: ClauseKey,
        variables: &mut VariableStore,
    ) -> Result<ClauseKey, ClauseStoreError> {
        match key {
            ClauseKey::Binary(_) => {
                log::error!(target: crate::log::targets::TRANSFER, "Attempt to transfer binary");
                Err(ClauseStoreError::TransferBinary)
            }
            ClauseKey::Formula(index) => {
                let formula_clause = &self.formula[index as usize];
                let copied_clause = formula_clause.literal_slice().to_vec();

                if copied_clause.len() != 2 {
                    log::error!(target: crate::log::targets::TRANSFER, "Attempt to transfer binary");
                    return Err(ClauseStoreError::TransferBinary);
                }

                let binary_key = self.new_binary_id()?;

                variables.remove_watch(unsafe { *copied_clause.get_unchecked(0) }, key)?;
                variables.remove_watch(unsafe { *copied_clause.get_unchecked(1) }, key)?;

                // as a new clause is created there's no need to add watches as in the learnt case

                let binary_clause = StoredClause::new_from(
                    binary_key,
                    copied_clause,
                    Vec::default(),
                    ClauseSource::Resolution,
                    variables,
                );

                self.binary.push(binary_clause);
                self.binary_graph.push(vec![key]);
                Ok(binary_key)
            }
            ClauseKey::Learned(_, _) => {
                let mut the_clause = self.remove_from_learned(key.index())?;

                if the_clause.len() != 2 {
                    log::error!(target: crate::log::targets::TRANSFER, "Attempt to transfer binary");
                    return Err(ClauseStoreError::TransferBinary);
                }

                let binary_key = self.new_binary_id()?;
                the_clause.key = binary_key;

                let watch_a = unsafe { *the_clause.get_unchecked(0) };
                let watch_b = unsafe { *the_clause.get_unchecked(1) };

                variables.remove_watch(watch_a, key)?;
                variables.remove_watch(watch_b, key)?;
                variables.add_watch(watch_a, WatchElement::Binary(watch_b, binary_key));
                variables.add_watch(watch_b, WatchElement::Binary(watch_a, binary_key));

                self.binary.push(the_clause);
                self.binary_graph.push(vec![key]);
                self.counts.learned += 1; // removing decrements the count

                Ok(binary_key)
            }
        }
    }

    // TODO: figure some improvement…
    pub fn reduce(&mut self, config: &Config) -> Result<(), ClauseStoreError> {
        let limit = self.counts.learned as usize / 2;
        'reduction_loop: for _ in 0..limit {
            if let Some(index) = self.learned_activity.peek_max() {
                let value = self.learned_activity.value_at(index);
                if value.lbd < config.glue_strength {
                    break 'reduction_loop;
                } else {
                    self.learned_activity.remove(index);
                    self.remove_from_learned(index)?;
                }
            } else {
                log::warn!(target: crate::log::targets::REDUCTION, "Reduction called but there were no candidates");
            }
        }
        log::debug!(target: crate::log::targets::REDUCTION, "Learnt clauses reduced to: {}", self.counts.learned);
        Ok(())
    }

    pub fn bump_activity(&mut self, key: ClauseKey, config: &Config) {
        let bump_activity = |s: &ActivityGlue| ActivityGlue {
            activity: s.activity + config::defaults::CLAUSE_BUMP,
            lbd: s.lbd,
        };

        let activity = self.learned_activity.value_at(key.index()).activity;
        if activity + self.learned_increment > ClauseActivity::MAX {
            let factor = 1.0 / activity;
            let decay_activity = |s: &ActivityGlue| ActivityGlue {
                activity: s.activity * factor,
                lbd: s.lbd,
            };
            self.learned_activity.apply_to_all(decay_activity);
            self.learned_increment *= factor
        }

        self.learned_activity
            .apply_to_index(key.index(), bump_activity);

        let decay = config.clause_decay * 1e-3;
        let factor = 1.0 / (1.0 - decay);
        self.learned_increment *= factor
    }
}

impl ClauseStore {
    /*
    Removing from learned checks to ensure removal is ok
    As the elements are optional for reuse, take places None at the index, as would be needed anyway
     */
    fn remove_from_learned(&mut self, index: usize) -> Result<StoredClause, ClauseStoreError> {
        if unsafe { self.learned.get_unchecked(index) }.is_none() {
            log::error!(target: crate::log::targets::CLAUSE_STORE, "attempt to remove something that is not there");
            Err(ClauseStoreError::MissingLearned)
        } else {
            // assert!(matches!(the_clause.key(), ClauseKey::LearnedLong(_, _)));
            let the_clause =
                std::mem::take(unsafe { self.learned.get_unchecked_mut(index) }).unwrap();
            self.learned_activity.remove(index);
            self.keys.push(the_clause.key());
            self.counts.learned -= 1;
            Ok(the_clause)
        }
    }
}