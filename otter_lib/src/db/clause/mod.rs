mod stored;
mod transfer;

use crossbeam::channel::Sender;
use stored::StoredClause;

use crate::{
    config::{self, Activity, Config, GlueStrength},
    db::{
        keys::{ClauseKey, FormulaIndex},
        variable::VariableDB,
    },
    dispatch::{
        delta::{self},
        report, Dispatch,
    },
    generic::heap::IndexHeap,
    misc::activity_glue::ActivityGlue,
    structures::{clause::Clause, literal::Literal},
    types::{
        clause::ClauseSource,
        err::{self},
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
    pub fn default(sender: &Sender<Dispatch>, config: &Config) -> Self {
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

            tx: sender.clone(),
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
    pub fn insert_clause(
        &mut self,
        source: ClauseSource,
        clause: Vec<Literal>,
        variables: &mut VariableDB,
    ) -> Result<ClauseKey, err::ClauseDB> {
        match clause.len() {
            0 => Err(err::ClauseDB::EmptyClause),
            1 => Err(err::ClauseDB::UnitClause),
            2 => {
                let key = self.new_binary_id()?;

                let delta = {
                    let clone = clause.clone();
                    match source {
                        ClauseSource::Formula => delta::ClauseDB::BinaryOriginal(key, clone),
                        ClauseSource::Resolution => delta::ClauseDB::BinaryResolution(key, clone),
                    }
                };
                self.tx.send(Dispatch::ClauseDB(delta));

                self.binary.push(StoredClause::from(key, clause, variables));

                Ok(key)
            }
            _ => match source {
                ClauseSource::Formula => {
                    let the_key = self.new_formula_id()?;

                    let delta = delta::ClauseDB::Original(the_key, clause.clone());
                    self.tx.send(Dispatch::ClauseDB(delta));

                    self.formula
                        .push(StoredClause::from(the_key, clause, variables));
                    Ok(the_key)
                }
                ClauseSource::Resolution => {
                    log::trace!(target: crate::log::targets::CLAUSE_DB, "Learning clause {}", clause.as_string());
                    self.counts.learned += 1;

                    let the_key = match self.empty_keys.len() {
                        0 => self.new_learned_id()?,
                        _ => self.empty_keys.pop().unwrap().retoken()?,
                    };

                    let delta = delta::ClauseDB::Learned(the_key, clause.clone());
                    self.tx.send(Dispatch::ClauseDB(delta));

                    let the_clause = StoredClause::from(the_key, clause, variables);

                    let value = ActivityGlue {
                        activity: Activity::default(),
                        lbd: the_clause.lbd(variables),
                    };

                    self.activity_heap.add(the_key.index(), value);
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

    // TODO: figure some improvement…
    // For example, before dropping a clause the lbd could be recalculated…
    pub fn reduce(&mut self) -> Result<(), err::ClauseDB> {
        let limit = self.learned.len() as usize / 2;

        'reduction_loop: for _ in 0..limit {
            if let Some(index) = self.activity_heap.peek_max() {
                let value = self.activity_heap.value_at(index);
                if value.lbd <= self.glue_strength {
                    break 'reduction_loop;
                } else {
                    self.activity_heap.remove(index);
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

    /*
    Removing from learned checks to ensure removal is ok
    As the elements are optional for reuse, take places None at the index, as would be needed anyway
     */
    fn remove_from_learned(&mut self, index: usize) -> Result<(), err::ClauseDB> {
        if unsafe { self.learned.get_unchecked(index) }.is_none() {
            log::error!(target: crate::log::targets::CLAUSE_DB, "attempt to remove something that is not there");
            Err(err::ClauseDB::MissingLearned)
        } else {
            // assert!(matches!(the_clause.key(), ClauseKey::LearnedLong(_, _)));
            let the_clause =
                std::mem::take(unsafe { self.learned.get_unchecked_mut(index) }).unwrap();

            self.tx.send(Dispatch::ClauseDB(delta::ClauseDB::Deletion(
                the_clause.key(),
                the_clause.to_vec(),
            )));

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

        self.activity_heap
            .apply_to_index(index as usize, bump_activity);

        let factor = 1.0 / (1.0 - self.activity_decay);
        self.activity_increment *= factor
    }

    // pub fn formula_clauses(&self) -> &[impl Clause] {
    //     &self.formula
    // }

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

    pub fn report_active(&self) {
        for clause in &self.binary {
            let report = report::ClauseDB::Active(clause.key(), clause.to_vec());
            self.tx.send(Dispatch::ClauseDBReport(report));
        }
        for clause in &self.formula {
            let report = report::ClauseDB::Active(clause.key(), clause.to_vec());
            self.tx.send(Dispatch::ClauseDBReport(report));
        }
        for clause in self.learned.iter().flatten() {
            if clause.is_active() {
                let report = report::ClauseDB::Active(clause.key(), clause.to_vec());
                self.tx.send(Dispatch::ClauseDBReport(report));
            }
        }
    }
}

use crate::assistants::resolution_buffer::BufErr;

impl ClauseDB {
    pub fn subsume(
        &mut self,
        key: ClauseKey,
        literal: Literal,
        variable_db: &mut VariableDB,
    ) -> Result<ClauseKey, BufErr> {
        let the_clause = self.get_carefully_mut(key).unwrap();
        match the_clause.len() {
            0..=2 => panic!("impossible"),
            3 => {
                let Ok(_) = the_clause.subsume(literal, variable_db, false) else {
                    return Err(BufErr::Subsumption);
                };
                let Ok(new_key) = self.transfer_to_binary(key, variable_db) else {
                    return Err(BufErr::Transfer);
                };
                Ok(new_key)
            }
            _ => {
                let Ok(_) = the_clause.subsume(literal, variable_db, true) else {
                    return Err(BufErr::Subsumption);
                };
                // TODO: Dispatches for subsumption…
                // let delta = delta::Resolution::Subsumed(key, literal);
                // self.tx.send(Dispatch::Resolution(delta));
                Ok(key)
            }
        }
    }
}
