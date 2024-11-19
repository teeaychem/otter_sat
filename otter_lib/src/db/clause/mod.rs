mod activity_glue;
mod stored;
mod transfer;

use crossbeam::channel::Sender;

use crate::{
    config::{self, Activity, Config, GlueStrength},
    db::{
        clause::{activity_glue::ActivityGlue, stored::StoredClause},
        keys::{ClauseKey, FormulaIndex},
        variable::VariableDB,
    },
    dispatch::{
        library::delta::{self, Delta},
        library::report::{self, Report},
        Dispatch,
    },
    generic::heap::IndexHeap,
    misc::log::targets::{self},
    structures::{clause::Clause, literal::Literal},
    types::{
        err::{self},
        gen::{self},
    },
};

pub enum ClauseKind {
    Binary,
    Long,
}

pub struct ClauseDB {
    counts: ClauseDBCounts,

    empty_keys: Vec<ClauseKey>,

    binary: Vec<StoredClause>,
    formula: Vec<StoredClause>,
    learned: Vec<Option<StoredClause>>,

    activity_heap: IndexHeap<ActivityGlue>,
    activity_increment: Activity,
    activity_decay: Activity,
    max_activity: Activity,
    glue_strength: GlueStrength,

    tx: Option<Sender<Dispatch>>,
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
    pub fn default(sender: Option<Sender<Dispatch>>, config: &Config) -> Self {
        ClauseDB {
            counts: ClauseDBCounts::default(),
            empty_keys: Vec::default(),

            formula: Vec::default(),
            learned: Vec::default(),
            binary: Vec::default(),

            activity_heap: IndexHeap::default(),
            activity_increment: Activity::default(),
            activity_decay: config.clause_decay * 1e-3,
            max_activity: config.activity_max,
            glue_strength: config.glue_strength,

            tx: sender,
        }
    }
}

impl ClauseDB {
    fn new_formula_id(&mut self) -> Result<ClauseKey, err::ClauseDB> {
        if self.counts.formula == FormulaIndex::MAX {
            return Err(err::ClauseDB::StorageExhausted);
        }
        let key = ClauseKey::Formula(self.counts.formula);
        self.counts.formula += 1;
        Ok(key)
    }

    fn new_binary_id(&mut self) -> Result<ClauseKey, err::ClauseDB> {
        if self.counts.binary == FormulaIndex::MAX {
            return Err(err::ClauseDB::StorageExhausted);
        }
        let key = ClauseKey::Binary(self.counts.binary);
        self.counts.binary += 1;
        Ok(key)
    }

    fn new_learned_id(&mut self) -> Result<ClauseKey, err::ClauseDB> {
        if self.learned.len() == FormulaIndex::MAX as usize {
            return Err(err::ClauseDB::StorageExhausted);
        }
        let key = ClauseKey::Learned(self.learned.len() as FormulaIndex, 0);
        Ok(key)
    }
}

impl ClauseDB {
    // pub fn get_carefully(&self, key: ClauseKey) -> Option<&StoredClause> {
    //     match key {
    //         ClauseKey::Formula(index) => self.formula.get(index as usize),
    //         ClauseKey::Binary(index) => self.binary.get(index as usize),
    //         ClauseKey::Learned(index, token) => match self.learned.get(index as usize) {
    //             Some(Some(clause)) => match clause.key() {
    //                 ClauseKey::Learned(_, clause_token) if clause_token == token => Some(clause),
    //                 _ => None,
    //             },
    //             _ => None,
    //         },
    //     }
    // }

    pub fn get(&self, key: ClauseKey) -> Result<&impl Clause, err::ClauseDB> {
        match key {
            ClauseKey::Formula(index) => unsafe { Ok(self.formula.get_unchecked(index as usize)) },
            ClauseKey::Binary(index) => unsafe { Ok(self.binary.get_unchecked(index as usize)) },
            ClauseKey::Learned(index, token) => unsafe {
                match self.learned.get_unchecked(index as usize) {
                    Some(clause) => match clause.key() {
                        ClauseKey::Learned(_, clause_token) if clause_token == token => Ok(clause),
                        _ => Err(err::ClauseDB::InvalidKeyToken),
                    },
                    None => Err(err::ClauseDB::InvalidKeyIndex),
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

    fn get_mut(&mut self, key: ClauseKey) -> Result<&mut StoredClause, err::ClauseDB> {
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
                        _ => Err(err::ClauseDB::InvalidKeyToken),
                    },
                    None => Err(err::ClauseDB::InvalidKeyIndex),
                }
            },
        }
    }
}

impl ClauseDB {
    pub fn note_use(&mut self, key: ClauseKey) {
        match key {
            ClauseKey::Learned(index, _) => {
                self.activity_heap.remove(index as usize);
            }
            ClauseKey::Formula(_) | ClauseKey::Binary(_) => {}
        }
    }

    pub fn reset_heap(&mut self) {
        for (index, slot) in self.learned.iter().enumerate() {
            if slot.is_some() {
                self.activity_heap.activate(index);
            }
        }
        self.activity_heap.reheap();
    }

    pub fn insert_clause(
        &mut self,
        source: gen::src::Clause,
        clause: Vec<Literal>,
        variables: &mut VariableDB,
    ) -> Result<ClauseKey, err::ClauseDB> {
        match clause.len() {
            0 => Err(err::ClauseDB::EmptyClause),
            1 => Err(err::ClauseDB::UnitClause),
            2 => {
                let key = self.new_binary_id()?;

                if let Some(tx) = &self.tx {
                    let delta = {
                        match source {
                            gen::src::Clause::Formula => {
                                delta::ClauseDB::BinaryOriginal(key, clause.clone())
                            }
                            gen::src::Clause::Resolution => {
                                delta::ClauseDB::BinaryResolution(key, clause.clone())
                            }
                        }
                    };
                    tx.send(Dispatch::Delta(Delta::ClauseDB(delta)));
                }

                self.binary.push(StoredClause::from(key, clause, variables));

                Ok(key)
            }
            _ => match source {
                gen::src::Clause::Formula => {
                    let the_key = self.new_formula_id()?;

                    if let Some(tx) = &self.tx {
                        let delta = delta::ClauseDB::Original(the_key, clause.clone());
                        tx.send(Dispatch::Delta(Delta::ClauseDB(delta)));
                    }

                    self.formula
                        .push(StoredClause::from(the_key, clause, variables));
                    Ok(the_key)
                }
                gen::src::Clause::Resolution => {
                    log::trace!(target: targets::CLAUSE_DB, "Learning clause {}", clause.as_string());
                    self.counts.learned += 1;

                    let the_key = match self.empty_keys.len() {
                        0 => self.new_learned_id()?,
                        _ => self.empty_keys.pop().unwrap().retoken()?,
                    };

                    if let Some(tx) = &self.tx {
                        let delta = delta::ClauseDB::Learned(the_key, clause.clone());
                        tx.send(Dispatch::Delta(Delta::ClauseDB(delta)));
                    }

                    let the_clause = StoredClause::from(the_key, clause, variables);

                    let value = ActivityGlue {
                        activity: 1.0,
                        lbd: the_clause.lbd(variables),
                    };

                    match the_key {
                        ClauseKey::Learned(_, 0) => {
                            self.activity_heap.add(the_key.index(), value);
                            self.activity_heap.activate(the_key.index());
                            self.learned.push(Some(the_clause));
                        }
                        ClauseKey::Learned(_, _) => unsafe {
                            self.activity_heap.revalue(the_key.index(), value);
                            self.activity_heap.activate(the_key.index());
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

    // TODO: figure some improvement…
    // For example, before dropping a clause the lbd could be recalculated…
    pub fn reduce(&mut self) -> Result<(), err::ClauseDB> {
        let count = self.learned.len();
        let limit = count / 2;

        // log::debug!(target: targets::REDUCTION, "Learnt A: {count}");
        // log::debug!(target: targets::REDUCTION, "Learnt B: {}", self.counts.learned);
        // log::debug!(target: targets::REDUCTION, "Learnt C: {}", self.learned.len() - self.empty_keys.len());
        // log::debug!(target: targets::REDUCTION, "Learnt D: {}", self.activity_heap.limit);

        'reduction_loop: for _ in 0..limit {
            if let Some(index) = self.activity_heap.peek_max() {
                let value = self.activity_heap.value_at(index);
                log::debug!(target: targets::REDUCTION, "Took: {:?}", value);
                if value.lbd <= self.glue_strength {
                    break 'reduction_loop;
                } else {
                    self.remove_from_learned(index)?;
                }
            } else {
                log::warn!(target: targets::REDUCTION, "Reduction called but there were no candidates");
            }
        }

        log::debug!(target: targets::REDUCTION, "Learnt clauses reduced from {count} to: {}", self.counts.learned);
        Ok(())
    }

    /*
    Removing from learned checks to ensure removal is ok
    As the elements are optional for reuse, take places None at the index, as would be needed anyway
     */
    fn remove_from_learned(&mut self, index: usize) -> Result<(), err::ClauseDB> {
        if unsafe { self.learned.get_unchecked(index) }.is_none() {
            log::error!(target: targets::CLAUSE_DB, "attempt to remove something that is not there");
            Err(err::ClauseDB::MissingLearned)
        } else {
            // assert!(matches!(the_clause.key(), ClauseKey::LearnedLong(_, _)));
            let the_clause =
                std::mem::take(unsafe { self.learned.get_unchecked_mut(index) }).unwrap();

            if let Some(tx) = &self.tx {
                let delta = delta::ClauseDB::Deletion(the_clause.key(), the_clause.to_vec());
                tx.send(Dispatch::Delta(Delta::ClauseDB(delta)));
            }

            self.activity_heap.remove(index);
            self.empty_keys.push(the_clause.key());
            self.counts.learned -= 1;
            Ok(())
        }
    }

    pub fn bump_activity(&mut self, index: FormulaIndex) {
        let bump_activity = |s: &ActivityGlue| ActivityGlue {
            activity: s.activity + config::defaults::CLAUSE_BUMP,
            lbd: s.lbd,
        };

        let activity = self.activity_heap.value_at(index as usize).activity;
        if activity + self.activity_increment > self.max_activity {
            let factor = 1.0 / activity;
            let decay_activity = |s: &ActivityGlue| ActivityGlue {
                activity: s.activity * factor,
                lbd: s.lbd,
            };
            self.activity_heap.apply_to_all(decay_activity);
            self.activity_increment *= factor
        }

        let index = index as usize;
        self.activity_heap.apply_to_index(index, bump_activity);
        self.activity_heap.heapify_if_active(index);

        let factor = 1.0 / (1.0 - self.activity_decay);
        self.activity_increment *= factor
    }

    pub fn clause_count(&self) -> usize {
        (self.counts.formula + self.counts.learned + self.counts.binary) as usize
    }

    pub fn all_clauses(&self) -> impl Iterator<Item = &impl Clause> + '_ {
        self.formula.iter().chain(
            self.binary.iter().chain(
                self.learned
                    .iter()
                    .flat_map(|maybe_clause| maybe_clause.as_ref()),
            ),
        )
    }

    pub fn dispatch_active(&self) {
        if let Some(tx) = &self.tx {
            for clause in &self.binary {
                let report = report::ClauseDB::Active(clause.key(), clause.to_vec());
                tx.send(Dispatch::Report(Report::ClauseDB(report)));
            }
            for clause in &self.formula {
                let report = report::ClauseDB::Active(clause.key(), clause.to_vec());
                tx.send(Dispatch::Report(Report::ClauseDB(report)));
            }
            for clause in self.learned.iter().flatten() {
                if clause.is_active() {
                    let report = report::ClauseDB::Active(clause.key(), clause.to_vec());
                    tx.send(Dispatch::Report(Report::ClauseDB(report)));
                }
            }
        }
    }
}

impl ClauseDB {
    /*
    If the resolved clause is binary then subsumption transfers the clause to the store for binary clauses
    This is safe to do as:
    - After backjumping all the observations at the current level will be forgotten
    - The clause does not appear in the observations of any previous stage
      + As, if the clause appeared in some previous stage then use of the clause would be a missed implication
      + And, missed implications are checked prior to conflicts
     */
    pub fn subsume(
        &mut self,
        key: ClauseKey,
        literal: Literal,
        variable_db: &mut VariableDB,
    ) -> Result<ClauseKey, err::RBuf> {
        let the_clause = self.get_carefully_mut(key).unwrap();
        match the_clause.len() {
            0..=2 => panic!("impossible"),
            3 => {
                let Ok(_) = the_clause.subsume(literal, variable_db, false) else {
                    return Err(err::RBuf::Subsumption);
                };
                let Ok(new_key) = self.transfer_to_binary(key, variable_db) else {
                    return Err(err::RBuf::Transfer);
                };
                Ok(new_key)
            }
            _ => {
                let Ok(_) = the_clause.subsume(literal, variable_db, true) else {
                    return Err(err::RBuf::Subsumption);
                };
                // TODO: Dispatches for subsumption…
                // let delta = delta::Resolution::Subsumed(key, literal);
                // self.tx.send(Dispatch::Resolution(delta));
                Ok(key)
            }
        }
    }
}
