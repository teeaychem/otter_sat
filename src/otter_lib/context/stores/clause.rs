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
        report::{self},
        Dispatch,
    },
    generic::heap::IndexHeap,
    structures::{
        clause::{stored::StoredClause, Clause},
        literal::Literal,
    },
    types::{
        clause::{ClauseSource, WatchElement},
        errs::{self},
    },
};

pub struct ClauseDB {
    counts: ClauseDBCounts,
    keys: Vec<ClauseKey>,
    formula: Vec<StoredClause>,

    learned: Vec<Option<StoredClause>>,

    learned_slots: FormulaIndex,

    binary: Vec<StoredClause>,

    learned_activity: IndexHeap<ActivityGlue>,
    learned_increment: ClauseActivity,

    tx: Sender<Dispatch>,
}

pub struct ClauseDBCounts {
    formula: FormulaIndex,
    binary: FormulaIndex,
    learned: FormulaIndex,
}

#[allow(clippy::derivable_impls)]
impl Default for ClauseDBCounts {
    fn default() -> Self {
        ClauseDBCounts {
            formula: 0,
            binary: 0,
            learned: 0,
        }
    }
}

impl ClauseDB {
    pub fn default(sender: &Sender<Dispatch>) -> Self {
        ClauseDB {
            counts: ClauseDBCounts::default(),
            keys: Vec::default(),
            formula: Vec::default(),
            learned: Vec::default(),
            learned_slots: 0,
            binary: Vec::default(),
            learned_activity: IndexHeap::default(),
            learned_increment: ClauseActivity::default(),
            tx: sender.clone(),
        }
    }
}

impl ClauseDB {
    fn new_formula_id(&mut self) -> Result<ClauseKey, errs::ClauseDB> {
        if self.counts.formula == FormulaIndex::MAX {
            return Err(errs::ClauseDB::StorageExhausted);
        }
        let key = ClauseKey::Formula(self.counts.formula);
        self.counts.formula += 1;
        Ok(key)
    }

    fn new_binary_id(&mut self) -> Result<ClauseKey, errs::ClauseDB> {
        if self.counts.binary == FormulaIndex::MAX {
            return Err(errs::ClauseDB::StorageExhausted);
        }
        let key = ClauseKey::Binary(self.counts.binary);
        self.counts.binary += 1;
        Ok(key)
    }

    fn new_learned_id(&mut self) -> Result<ClauseKey, errs::ClauseDB> {
        if self.learned_slots == FormulaIndex::MAX {
            return Err(errs::ClauseDB::StorageExhausted);
        }
        let key = ClauseKey::Learned(self.learned_slots, 0);
        self.learned_slots += 1;
        Ok(key)
    }
}

impl ClauseDB {
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

    pub fn get(&self, key: ClauseKey) -> Result<&StoredClause, errs::ClauseDB> {
        match key {
            ClauseKey::Formula(index) => unsafe { Ok(self.formula.get_unchecked(index as usize)) },
            ClauseKey::Binary(index) => unsafe { Ok(self.binary.get_unchecked(index as usize)) },
            ClauseKey::Learned(index, token) => unsafe {
                match self.learned.get_unchecked(index as usize) {
                    Some(clause) => match clause.key() {
                        ClauseKey::Learned(_, clause_token) if clause_token == token => Ok(clause),
                        _ => Err(errs::ClauseDB::InvalidKeyToken),
                    },
                    None => Err(errs::ClauseDB::InvalidKeyIndex),
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

    pub fn get_mut(&mut self, key: ClauseKey) -> Result<&mut StoredClause, errs::ClauseDB> {
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
                        _ => Err(errs::ClauseDB::InvalidKeyToken),
                    },
                    None => Err(errs::ClauseDB::InvalidKeyIndex),
                }
            },
        }
    }
}

impl ClauseDB {
    pub fn insert_clause(
        &mut self,
        source: ClauseSource,
        clause: Vec<Literal>,
        variables: &mut VariableStore,
        resolution_keys: Vec<ClauseKey>,
        config: &Config,
    ) -> Result<ClauseKey, errs::ClauseDB> {
        match clause.len() {
            0 => Err(errs::ClauseDB::EmptyClause),
            1 => Err(errs::ClauseDB::UnitClause),
            2 => {
                let the_key = self.new_binary_id()?;

                match source {
                    ClauseSource::Formula => {
                        let delta = delta::ClauseDB::BinaryFormula(the_key, clause.clone());
                        self.tx.send(Dispatch::ClauseDB(delta))
                    }
                    ClauseSource::Resolution => {
                        let delta = delta::ClauseDB::BinaryResolution(the_key, clause.clone());
                        self.tx.send(Dispatch::ClauseDB(delta))
                    }
                };

                self.binary
                    .push(StoredClause::from(the_key, clause, variables));

                Ok(the_key)
            }
            _ => match source {
                ClauseSource::Formula => {
                    let the_key = self.new_formula_id()?;

                    let delta = delta::ClauseDB::Formula(the_key, clause.clone());
                    self.tx.send(Dispatch::ClauseDB(delta));

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

                    let delta = delta::ClauseDB::Learned(the_key, clause.clone());
                    self.tx.send(Dispatch::ClauseDB(delta));

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

    /*
    To keep things simple a formula clause is ignored while a learnt clause is deleted

    */
    pub fn transfer_to_binary(
        &mut self,
        key: ClauseKey,
        variables: &mut VariableStore,
    ) -> Result<ClauseKey, errs::ClauseDB> {
        match key {
            ClauseKey::Binary(_) => {
                log::error!(target: crate::log::targets::TRANSFER, "Attempt to transfer binary");
                Err(errs::ClauseDB::TransferBinary)
            }
            ClauseKey::Formula(index) | ClauseKey::Learned(index, _) => {
                let the_clause = self.get_mut(key)?;
                the_clause.deactivate();
                let copied_clause = the_clause.to_vec();

                if copied_clause.len() != 2 {
                    log::error!(target: crate::log::targets::TRANSFER, "Attempt to transfer binary");
                    return Err(errs::ClauseDB::TransferBinary);
                }

                let b_key = self.new_binary_id()?;

                let delta = delta::ClauseDB::TransferBinary(key, b_key, copied_clause.clone());
                self.tx.send(Dispatch::ClauseDB(delta));

                variables.remove_watch(unsafe { copied_clause.get_unchecked(0) }, key)?;
                variables.remove_watch(unsafe { copied_clause.get_unchecked(1) }, key)?;

                let binary_clause = StoredClause::from(b_key, copied_clause, variables);

                self.binary.push(binary_clause);

                if matches!(key, ClauseKey::Learned(_, _)) {
                    self.remove_from_learned(key.index())?;
                    self.counts.learned += 1; // removing decrements the coun
                }

                Ok(b_key)
            }
        }
    }
}

impl ClauseDB {
    // TODO: figure some improvement…
    // For example, before dropping a clause the lbd could be recalculated…
    pub fn reduce(&mut self, config: &Config) -> Result<(), errs::ClauseDB> {
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

impl ClauseDB {
    /*
    Removing from learned checks to ensure removal is ok
    As the elements are optional for reuse, take places None at the index, as would be needed anyway
     */
    fn remove_from_learned(&mut self, index: usize) -> Result<(), errs::ClauseDB> {
        if unsafe { self.learned.get_unchecked(index) }.is_none() {
            log::error!(target: crate::log::targets::CLAUSE_STORE, "attempt to remove something that is not there");
            Err(errs::ClauseDB::MissingLearned)
        } else {
            // assert!(matches!(the_clause.key(), ClauseKey::LearnedLong(_, _)));
            let the_clause =
                std::mem::take(unsafe { self.learned.get_unchecked_mut(index) }).unwrap();

            self.tx.send(Dispatch::ClauseDB(delta::ClauseDB::Deletion(
                the_clause.key(),
                the_clause.to_vec(),
            )));

            self.learned_activity.remove(index);
            self.keys.push(the_clause.key());
            self.counts.learned -= 1;
            Ok(())
        }
    }
}

impl ClauseDB {
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

    pub fn formula_clauses(&self) -> &[StoredClause] {
        &self.formula
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
}
