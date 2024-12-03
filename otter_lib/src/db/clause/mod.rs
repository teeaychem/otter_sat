mod activity_glue;
mod stored;
mod transfer;

use std::rc::Rc;

use crate::{
    config::{dbs::ClauseDBConfig, Config},
    db::{
        clause::{activity_glue::ActivityGlue, stored::dbClause},
        keys::{ClauseKey, FormulaIndex},
        variable::VariableDB,
    },
    dispatch::{
        library::{
            delta::{self, Delta},
            report::{self, Report},
        },
        Dispatch,
    },
    generic::heap::IndexHeap,
    misc::log::targets::{self},
    structures::{
        clause::{Clause, ClauseT},
        literal::Literal,
    },
    types::{
        err::{self},
        gen::{self},
    },
};

pub enum ClauseKind {
    Unit,
    Binary,
    Long,
}

pub struct ClauseDB {
    counts: ClauseDBCounts,

    empty_keys: Vec<ClauseKey>,

    pub unit: Vec<Literal>,
    binary: Vec<dbClause>,
    formula: Vec<dbClause>,
    learned: Vec<Option<dbClause>>,

    activity_heap: IndexHeap<ActivityGlue>,

    dispatcher: Option<Rc<dyn Fn(Dispatch)>>,
    config: ClauseDBConfig,
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
    pub fn new(config: &Config, dispatcher: Option<Rc<dyn Fn(Dispatch)>>) -> Self {
        ClauseDB {
            counts: ClauseDBCounts::default(),
            empty_keys: Vec::default(),

            unit: Vec::default(),
            formula: Vec::default(),
            learned: Vec::default(),
            binary: Vec::default(),

            activity_heap: IndexHeap::default(),
            config: config.clause_db.clone(),

            dispatcher,
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

    pub fn get(&self, key: ClauseKey) -> Result<&impl ClauseT, err::ClauseDB> {
        match key {
            ClauseKey::Unit(_) => Err(err::ClauseDB::GetUnitKey),
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

    pub fn get_carefully_mut(&mut self, key: ClauseKey) -> Option<&mut dbClause> {
        match key {
            ClauseKey::Unit(l) => None,
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

    fn get_mut(&mut self, key: ClauseKey) -> Result<&mut dbClause, err::ClauseDB> {
        match key {
            ClauseKey::Unit(_) => Err(err::ClauseDB::GetUnitKey),
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
            ClauseKey::Unit(_) | ClauseKey::Binary(_) | ClauseKey::Formula(_) => {}
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

    /// Stores a clause with an automatically generated id.
    /// In order to use the clause the watch literals of the struct must be initialised.
    pub fn store(
        &mut self,
        clause: impl ClauseT,
        source: gen::src::Clause,
        variables: &mut VariableDB,
    ) -> Result<ClauseKey, err::ClauseDB> {
        match clause.size() {
            0 => Err(err::ClauseDB::EmptyClause),

            1 => {
                let the_literal = clause.literals().next().expect("condition already checked");
                self.unit.push(*the_literal);
                Ok(ClauseKey::Unit(*the_literal))
            }

            2 => {
                let key = self.new_binary_id()?;

                if let Some(dispatcher) = &self.dispatcher {
                    let delta = delta::ClauseDB::ClauseStart;
                    dispatcher(Dispatch::Delta(Delta::ClauseDB(delta)));
                    for literal in clause.literals() {
                        let delta = delta::ClauseDB::ClauseLiteral(*literal);
                        dispatcher(Dispatch::Delta(Delta::ClauseDB(delta)));
                    }
                    let delta = {
                        match source {
                            gen::src::Clause::Original => delta::ClauseDB::BinaryOriginal(key),
                            gen::src::Clause::Resolution => delta::ClauseDB::BinaryResolution(key),
                        }
                    };
                    dispatcher(Dispatch::Delta(Delta::ClauseDB(delta)));
                }

                self.binary
                    .push(dbClause::from(key, clause.transform_to_vec(), variables));

                Ok(key)
            }

            _ => match source {
                gen::src::Clause::Original => {
                    let the_key = self.new_formula_id()?;

                    if let Some(dispatcher) = &self.dispatcher {
                        let delta = delta::ClauseDB::ClauseStart;
                        dispatcher(Dispatch::Delta(Delta::ClauseDB(delta)));
                        for literal in clause.literals() {
                            let delta = delta::ClauseDB::ClauseLiteral(*literal);
                            dispatcher(Dispatch::Delta(Delta::ClauseDB(delta)));
                        }
                        let delta = delta::ClauseDB::Original(the_key);
                        dispatcher(Dispatch::Delta(Delta::ClauseDB(delta)));
                    }

                    self.formula.push(dbClause::from(
                        the_key,
                        clause.transform_to_vec(),
                        variables,
                    ));
                    Ok(the_key)
                }

                gen::src::Clause::Resolution => {
                    log::trace!(target: targets::CLAUSE_DB, "Learning clause {}", clause.as_string());
                    self.counts.learned += 1;

                    let the_key = match self.empty_keys.len() {
                        0 => self.new_learned_id()?,
                        _ => self.empty_keys.pop().unwrap().retoken()?,
                    };

                    if let Some(dispatcher) = &self.dispatcher {
                        let delta = delta::ClauseDB::ClauseStart;
                        dispatcher(Dispatch::Delta(Delta::ClauseDB(delta)));
                        for literal in clause.literals() {
                            let delta = delta::ClauseDB::ClauseLiteral(*literal);
                            dispatcher(Dispatch::Delta(Delta::ClauseDB(delta)));
                        }
                        let delta = delta::ClauseDB::Resolution(the_key);
                        dispatcher(Dispatch::Delta(Delta::ClauseDB(delta)));
                    }

                    let the_clause = dbClause::from(the_key, clause.transform_to_vec(), variables);

                    let value = ActivityGlue {
                        activity: 1.0,
                        lbd: the_clause.lbd(variables),
                    };

                    self.activity_heap.add(the_key.index(), value);
                    self.activity_heap.activate(the_key.index());

                    match the_key {
                        ClauseKey::Learned(_, 0) => self.learned.push(Some(the_clause)),
                        ClauseKey::Learned(_, _) => unsafe {
                            *self.learned.get_unchecked_mut(the_key.index()) = Some(the_clause)
                        },
                        _ => panic!("not possible"),
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
                if value.lbd <= self.config.lbd_bound {
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

            if let Some(dispatcher) = &self.dispatcher {
                let delta = delta::ClauseDB::ClauseStart;
                dispatcher(Dispatch::Delta(Delta::ClauseDB(delta)));
                for literal in the_clause.literals() {
                    let delta = delta::ClauseDB::ClauseLiteral(*literal);
                    dispatcher(Dispatch::Delta(Delta::ClauseDB(delta)));
                }
                let delta = delta::ClauseDB::Deletion(the_clause.key());
                dispatcher(Dispatch::Delta(Delta::ClauseDB(delta)));
            }

            self.activity_heap.remove(index);
            self.empty_keys.push(the_clause.key());
            self.counts.learned -= 1;
            Ok(())
        }
    }

    pub fn bump_activity(&mut self, index: FormulaIndex) {
        if let Some(max) = self.activity_heap.peek_max_value() {
            if max.activity + self.config.bump > self.config.max_bump {
                let factor = 1.0 / max.activity;
                let decay_activity = |s: &ActivityGlue| ActivityGlue {
                    activity: s.activity * factor,
                    lbd: s.lbd,
                };
                self.activity_heap.apply_to_all(decay_activity);
                self.config.bump *= factor
            }
        }

        let bump_activity = |s: &ActivityGlue| ActivityGlue {
            activity: s.activity + self.config.bump,
            lbd: s.lbd,
        };

        let index = index as usize;
        self.activity_heap.apply_to_index(index, bump_activity);
        self.activity_heap.heapify_if_active(index);

        self.config.bump *= 1.0 / (1.0 - self.config.decay);
    }

    pub fn clause_count(&self) -> usize {
        (self.counts.formula + self.counts.learned + self.counts.binary) as usize
    }

    pub fn all_unit_clauses(&self) -> impl Iterator<Item = &Literal> {
        self.unit.iter()
    }

    pub fn all_nonunit_clauses(&self) -> impl Iterator<Item = &impl ClauseT> + '_ {
        self.formula.iter().chain(
            self.binary.iter().chain(
                self.learned
                    .iter()
                    .flat_map(|maybe_clause| maybe_clause.as_ref()),
            ),
        )
    }

    pub fn dispatch_active(&self) {
        if let Some(dispatcher) = &self.dispatcher {
            for clause in &self.binary {
                let report = report::ClauseDB::Active(clause.key(), clause.to_vec());
                dispatcher(Dispatch::Report(Report::ClauseDB(report)));
            }
            for clause in &self.formula {
                let report = report::ClauseDB::Active(clause.key(), clause.to_vec());
                dispatcher(Dispatch::Report(Report::ClauseDB(report)));
            }
            for clause in self.learned.iter().flatten() {
                if clause.is_active() {
                    let report = report::ClauseDB::Active(clause.key(), clause.to_vec());
                    dispatcher(Dispatch::Report(Report::ClauseDB(report)));
                }
            }
            for literal in self.all_unit_clauses() {
                let report = report::LiteralDB::Active(*literal);
                dispatcher(Dispatch::Report(report::Report::LiteralDB(report)));
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
    ) -> Result<ClauseKey, err::ResolutionBuffer> {
        let the_clause = self.get_carefully_mut(key).unwrap();
        match the_clause.len() {
            0..=2 => panic!("impossible"),
            3 => {
                let Ok(_) = the_clause.subsume(literal, variable_db, false) else {
                    return Err(err::ResolutionBuffer::Subsumption);
                };
                let Ok(new_key) = self.transfer_to_binary(key, variable_db) else {
                    return Err(err::ResolutionBuffer::Transfer);
                };
                Ok(new_key)
            }
            _ => {
                let Ok(_) = the_clause.subsume(literal, variable_db, true) else {
                    return Err(err::ResolutionBuffer::Subsumption);
                };
                // TODO: Dispatches for subsumption…
                // let delta = delta::Resolution::Subsumed(key, literal);
                // (Dispatch::Resolution(delta));
                Ok(key)
            }
        }
    }
}
