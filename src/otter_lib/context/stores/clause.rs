use std::ops::Deref;

use crossbeam::channel::Sender;

use crate::{
    config::{self, ClauseActivity, Config},
    context::stores::{
        activity_glue::ActivityGlue, variable::VariableStore, ClauseKey, FormulaIndex,
    },
    dispatch::{
        self,
        delta::{self},
        Dispatch,
    },
    generic::heap::IndexHeap,
    structures::{
        clause::{stored::StoredClause, Clause},
        literal::Literal,
    },
    types::{
        clause::{ClauseSource, WatchElement},
        errs::ClauseStoreErr,
    },
};

pub struct ClauseStore {
    counts: ClauseStoreCounts,
    keys: Vec<ClauseKey>,
    formula: Vec<StoredClause>,

    learned: Vec<Option<StoredClause>>,

    learned_slots: FormulaIndex,

    binary: Vec<StoredClause>,

    learned_activity: IndexHeap<ActivityGlue>,
    learned_increment: ClauseActivity,

    sender: Sender<Dispatch>,
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

impl ClauseStore {
    pub fn default(sender: &Sender<Dispatch>) -> Self {
        ClauseStore {
            counts: ClauseStoreCounts::default(),
            keys: Vec::default(),
            formula: Vec::default(),
            learned: Vec::default(),
            learned_slots: 0,
            binary: Vec::default(),
            learned_activity: IndexHeap::default(),
            learned_increment: ClauseActivity::default(),
            sender: sender.clone(),
        }
    }
}

impl ClauseStore {
    fn new_formula_id(&mut self) -> Result<ClauseKey, ClauseStoreErr> {
        if self.counts.formula == FormulaIndex::MAX {
            return Err(ClauseStoreErr::StorageExhausted);
        }
        let key = ClauseKey::Formula(self.counts.formula);
        self.counts.formula += 1;
        Ok(key)
    }

    fn new_binary_id(&mut self) -> Result<ClauseKey, ClauseStoreErr> {
        if self.counts.binary == FormulaIndex::MAX {
            return Err(ClauseStoreErr::StorageExhausted);
        }
        let key = ClauseKey::Binary(self.counts.binary);
        self.counts.binary += 1;
        Ok(key)
    }

    fn new_learned_id(&mut self) -> Result<ClauseKey, ClauseStoreErr> {
        if self.learned_slots == FormulaIndex::MAX {
            return Err(ClauseStoreErr::StorageExhausted);
        }
        let key = ClauseKey::Learned(self.learned_slots, 0);
        self.learned_slots += 1;
        Ok(key)
    }
}

impl ClauseStore {
    pub fn formula_clauses(&self) -> &[StoredClause] {
        &self.formula
    }

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

    pub fn get(&self, key: ClauseKey) -> Result<&StoredClause, ClauseStoreErr> {
        match key {
            ClauseKey::Formula(index) => unsafe { Ok(self.formula.get_unchecked(index as usize)) },
            ClauseKey::Binary(index) => unsafe { Ok(self.binary.get_unchecked(index as usize)) },
            ClauseKey::Learned(index, token) => unsafe {
                match self.learned.get_unchecked(index as usize) {
                    Some(clause) => match clause.key() {
                        ClauseKey::Learned(_, clause_token) if clause_token == token => Ok(clause),
                        _ => Err(ClauseStoreErr::InvalidKeyToken),
                    },
                    None => Err(ClauseStoreErr::InvalidKeyIndex),
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

    pub fn get_mut(&mut self, key: ClauseKey) -> Result<&mut StoredClause, ClauseStoreErr> {
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
                        _ => Err(ClauseStoreErr::InvalidKeyToken),
                    },
                    None => Err(ClauseStoreErr::InvalidKeyIndex),
                }
            },
        }
    }

    pub fn insert_clause(
        &mut self,
        source: ClauseSource,
        clause: Vec<Literal>,
        variables: &mut VariableStore,
        resolution_keys: Vec<ClauseKey>,
        config: &Config,
    ) -> Result<ClauseKey, ClauseStoreErr> {
        match clause.len() {
            0 => Err(ClauseStoreErr::EmptyClause),
            1 => Err(ClauseStoreErr::UnitClause),
            2 => {
                let the_key = self.new_binary_id()?;

                match source {
                    ClauseSource::Formula => {
                        self.sender
                            .send(Dispatch::ClauseDB(delta::ClauseStore::BinaryFormula(
                                the_key,
                                clause.clone(),
                            )))
                    }
                    ClauseSource::Resolution => {
                        self.sender
                            .send(Dispatch::ClauseDB(delta::ClauseStore::BinaryResolution(
                                the_key,
                                clause.clone(),
                            )))
                    }
                };

                self.binary
                    .push(StoredClause::from(the_key, clause, variables));

                Ok(the_key)
            }
            _ => match source {
                ClauseSource::Formula => {
                    let the_key = self.new_formula_id()?;

                    self.sender
                        .send(Dispatch::ClauseDB(delta::ClauseStore::Formula(
                            the_key,
                            clause.clone(),
                        )));

                    self.formula
                        .push(StoredClause::from(the_key, clause, variables));
                    Ok(the_key)
                }
                ClauseSource::Resolution => {
                    log::trace!(target: crate::log::targets::CLAUSE_STORE, "Learning clause {}", clause.as_string());
                    self.counts.learned += 1;

                    let the_key = match self.keys.len() {
                        0 => self.new_learned_id()?,
                        _ => self.keys.pop().unwrap().retoken()?,
                    };

                    self.sender
                        .send(Dispatch::ClauseDB(delta::ClauseStore::Learned(
                            the_key,
                            clause.clone(),
                        )));

                    let the_clause = StoredClause::from(the_key, clause, variables);

                    let value = ActivityGlue {
                        activity: ClauseActivity::default(),
                        lbd: the_clause.lbd(variables),
                    };

                    self.learned_activity.insert(the_key.index(), value);
                    match the_key {
                        ClauseKey::Learned(_, 0) => {
                            self.learned.push(Some(the_clause));
                        }
                        ClauseKey::Learned(_, _) => unsafe {
                            *self.learned.get_unchecked_mut(the_key.index()) = Some(the_clause);
                        },
                        _ => panic!("X"),
                    };

                    Ok(the_key)
                }
            },
        }
    }

    pub fn clause_count(&self) -> usize {
        (self.counts.formula + self.counts.learned + self.counts.binary) as usize
    }

    pub fn all_clauses(&self) -> impl Iterator<Item = &StoredClause> + '_ {
        self.formula.iter().chain(
            self.binary.iter().chain(
                self.learned
                    .iter()
                    .flat_map(|maybe_clause| maybe_clause.as_ref()),
            ),
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
    ) -> Result<ClauseKey, ClauseStoreErr> {
        match key {
            ClauseKey::Binary(_) => {
                log::error!(target: crate::log::targets::TRANSFER, "Attempt to transfer binary");
                Err(ClauseStoreErr::TransferBinary)
            }
            ClauseKey::Formula(index) => {
                let formula_clause = &mut self.formula[index as usize];
                formula_clause.deactivate();
                let copied_clause = formula_clause.deref().to_vec();

                if copied_clause.len() != 2 {
                    log::error!(target: crate::log::targets::TRANSFER, "Attempt to transfer binary");
                    return Err(ClauseStoreErr::TransferBinary);
                }

                let formula_key = formula_clause.key();
                let binary_key = self.new_binary_id()?;

                // TODO: May need to note the original formula
                self.sender.send(Dispatch::ClauseDB(
                    dispatch::delta::ClauseStore::TransferFormula(formula_key, binary_key),
                ));

                variables.remove_watch(unsafe { copied_clause.get_unchecked(0) }, key)?;
                variables.remove_watch(unsafe { copied_clause.get_unchecked(1) }, key)?;

                // as a new clause is created there's no need to add watches as in the learnt case

                let binary_clause = StoredClause::from(binary_key, copied_clause, variables);

                self.binary.push(binary_clause);
                Ok(binary_key)
            }
            ClauseKey::Learned(_, _) => {
                let mut the_clause = self.remove_from_learned(key.index())?;

                if the_clause.len() != 2 {
                    log::error!(target: crate::log::targets::TRANSFER, "Attempt to transfer binary");
                    return Err(ClauseStoreErr::TransferBinary);
                }

                let binary_key = self.new_binary_id()?;

                self.sender.send(Dispatch::ClauseDB(
                    dispatch::delta::ClauseStore::TransferLearned(the_clause.key(), binary_key),
                ));

                the_clause.replace_key(binary_key);

                let watch_a = unsafe { the_clause.get_unchecked(0) };
                let watch_b = unsafe { the_clause.get_unchecked(1) };

                variables.remove_watch(watch_a, key)?;
                variables.remove_watch(watch_b, key)?;
                variables.add_watch(watch_a, WatchElement::Binary(*watch_b, binary_key));
                variables.add_watch(watch_b, WatchElement::Binary(*watch_a, binary_key));

                self.binary.push(the_clause);
                self.counts.learned += 1; // removing decrements the count

                Ok(binary_key)
            }
        }
    }

    // TODO: figure some improvement…
    // For example, before dropping a clause the lbd could be recalculated…
    pub fn reduce(&mut self, config: &Config) -> Result<(), ClauseStoreErr> {
        let limit = self.counts.learned as usize / 2;

        'reduction_loop: for _ in 0..limit {
            if let Some(index) = self.learned_activity.peek_max() {
                let value = self.learned_activity.value_at(index);
                if value.lbd <= config.glue_strength {
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

    pub fn bump_activity(&mut self, index: FormulaIndex, config: &Config) {
        let bump_activity = |s: &ActivityGlue| ActivityGlue {
            activity: s.activity + config::defaults::CLAUSE_BUMP,
            lbd: s.lbd,
        };

        let activity = self.learned_activity.value_at(index as usize).activity;
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
            .apply_to_index(index as usize, bump_activity);

        let decay = config.clause_decay * 1e-3;
        let factor = 1.0 / (1.0 - decay);
        self.learned_increment *= factor
    }

    // pub fn source(&self, key: ClauseKey) -> &[ClauseKey] {
    //     match key {
    //         ClauseKey::Formula(_) => &[],
    //         ClauseKey::Binary(index) => &self.binary_graph[index as usize],
    //         ClauseKey::Learned(index, token) => {
    //             &self.resolution_graph[index as usize][token as usize]
    //         }
    //     }
    // }
}

impl ClauseStore {
    /*
    Removing from learned checks to ensure removal is ok
    As the elements are optional for reuse, take places None at the index, as would be needed anyway
     */
    fn remove_from_learned(&mut self, index: usize) -> Result<StoredClause, ClauseStoreErr> {
        if unsafe { self.learned.get_unchecked(index) }.is_none() {
            log::error!(target: crate::log::targets::CLAUSE_STORE, "attempt to remove something that is not there");
            Err(ClauseStoreErr::MissingLearned)
        } else {
            // assert!(matches!(the_clause.key(), ClauseKey::LearnedLong(_, _)));
            let the_clause =
                std::mem::take(unsafe { self.learned.get_unchecked_mut(index) }).unwrap();

            self.sender
                .send(Dispatch::ClauseDB(dispatch::delta::ClauseStore::Deletion(
                    the_clause.key(),
                )));

            self.learned_activity.remove(index);
            self.keys.push(the_clause.key());
            self.counts.learned -= 1;
            Ok(the_clause)
        }
    }
}
