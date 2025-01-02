pub mod activity_glue;
pub mod db_clause;
mod transfer;

use std::{borrow::Borrow, rc::Rc};

use db_clause::dbClause;

use crate::{
    config::{dbs::ClauseDBConfig, Config},
    db::{
        atom::AtomDB,
        clause::activity_glue::ActivityLBD,
        keys::{ClauseKey, FormulaIndex},
    },
    dispatch::{
        library::{
            delta::{self, Delta},
            report::{self, Report},
        },
        Dispatch,
    },
    generic::index_heap::IndexHeap,
    misc::log::targets::{self},
    structures::{
        clause::{Clause, Source},
        literal::abLiteral,
    },
    types::err::{self},
};

/// A database of clause related things.
///
/// Records of clauses are distinguished by a mix of [kind](crate::structures::clause::ClauseKind) and/or [source](crate::structures::clause::Source).
///
/// Fields of the database are private to ensure the use of methods which may be needed to uphold invariants.
pub struct ClauseDB {
    /// Clause database specific configuration parameters.
    config: ClauseDBConfig,
    /// A count of addition clauses.
    // This can't be inferred from the learned vec, as indicies may be reused.
    addition_count: usize,
    /// A stack of keys for learned clauses whose indicies are empty.
    empty_keys: Vec<ClauseKey>,

    /// Unit clause, stored as literals.
    unit: Vec<abLiteral>,
    /// Binary clauses.
    binary: Vec<dbClause>,
    /// Original clauses.
    original: Vec<dbClause>,
    /// Addition clauses.
    addition: Vec<Option<dbClause>>,

    /// An activity heap of addition clause keys.
    activity_heap: IndexHeap<ActivityLBD>,
    /// Where to send dispatches.
    dispatcher: Option<Rc<dyn Fn(Dispatch)>>,
}

impl ClauseDB {
    pub fn new(config: &Config, dispatcher: Option<Rc<dyn Fn(Dispatch)>>) -> Self {
        ClauseDB {
            addition_count: 0,
            empty_keys: Vec::default(),

            unit: Vec::default(),
            original: Vec::default(),
            addition: Vec::default(),
            binary: Vec::default(),

            activity_heap: IndexHeap::default(),
            config: config.clause_db.clone(),

            dispatcher,
        }
    }
}

/// Methods for storing clauses.
///
/// As key generation is local, the methods are not exported.
/// Though, note, as keys use a [index](FormulaIndex) which may be smaller than [usize] a check is made to ensure it will be possible to generate the key.
impl ClauseDB {
    /// A key to a binary clause.
    fn fresh_binary_key(&mut self) -> Result<ClauseKey, err::ClauseDB> {
        if self.binary.len() == FormulaIndex::MAX as usize {
            return Err(err::ClauseDB::StorageExhausted);
        }
        let key = ClauseKey::Binary(self.binary.len() as FormulaIndex);
        Ok(key)
    }

    /// A key to an original clause.
    fn fresh_original_key(&mut self) -> Result<ClauseKey, err::ClauseDB> {
        if self.original.len() == FormulaIndex::MAX as usize {
            return Err(err::ClauseDB::StorageExhausted);
        }
        let key = ClauseKey::Original(self.original.len() as FormulaIndex);
        Ok(key)
    }

    /// A key to an addition clause.
    fn fresh_addition_key(&mut self) -> Result<ClauseKey, err::ClauseDB> {
        if self.addition.len() == FormulaIndex::MAX as usize {
            return Err(err::ClauseDB::StorageExhausted);
        }
        let key = ClauseKey::Addition(self.addition.len() as FormulaIndex, 0);
        Ok(key)
    }

    /// Stores a clause with an automatically generated id.
    ///
    /// ```rust, ignore
    /// self.clause_db.store(clause, ClauseSource::Resolution, &mut self.atom_db);
    /// ```
    ///
    /// Any instance of storing a clause should directly or indirectly use this method, as it maintains the activity heap, watched literals, etc.
    pub fn store(
        &mut self,
        clause: impl Clause,
        source: Source,
        atoms: &mut AtomDB,
    ) -> Result<ClauseKey, err::ClauseDB> {
        match clause.size() {
            0 => Err(err::ClauseDB::EmptyClause),

            1 => {
                // The match ensures there is a next (and then no further) literal in the clause.
                let the_literal = unsafe { *clause.literals().next().unwrap_unchecked() };

                self.unit.push(the_literal);
                Ok(ClauseKey::Unit(the_literal))
            }

            2 => {
                let key = self.fresh_binary_key()?;

                self.binary
                    .push(dbClause::from(key, clause.canonical(), atoms));

                Ok(key)
            }

            _ => match source {
                Source::BCP | Source::FreeChoice => panic!("!"), // Sources only valid for unit clauses.

                Source::Original => {
                    let key = self.fresh_original_key()?;
                    let stored_form = dbClause::from(key, clause.canonical(), atoms);

                    self.original.push(stored_form);
                    Ok(key)
                }

                Source::Resolution => {
                    log::trace!(target: targets::CLAUSE_DB, "Learning clause {}", clause.as_string());
                    self.addition_count += 1;

                    let the_key = match self.empty_keys.len() {
                        0 => self.fresh_addition_key()?,
                        _ => self.empty_keys.pop().unwrap().retoken()?,
                    };

                    let stored_form = dbClause::from(the_key, clause.canonical(), atoms);

                    let value = ActivityLBD {
                        activity: 1.0,
                        lbd: stored_form.lbd(atoms),
                    };

                    self.activity_heap.add(the_key.index(), value);
                    self.activity_heap.activate(the_key.index());

                    match the_key {
                        ClauseKey::Addition(_, 0) => self.addition.push(Some(stored_form)),
                        ClauseKey::Addition(_, _) => unsafe {
                            *self.addition.get_unchecked_mut(the_key.index()) = Some(stored_form)
                        },
                        _ => panic!("not possible"),
                    };

                    Ok(the_key)
                }
            },
        }
    }
}

/// Methods to get clauses stored in the database.
impl ClauseDB {
    /// Returns Ok(clause) corresponding to the given key, or an Err(issue) otherwise.
    ///
    /// ```rust, ignore
    /// self.clause_db.get_db_clause(&key)?
    /// ```
    pub fn get_db_clause(&self, key: &ClauseKey) -> Result<&dbClause, err::ClauseDB> {
        match key {
            ClauseKey::Unit(_) => Err(err::ClauseDB::GetUnitKey),
            ClauseKey::Original(index) => {
                //
                match self.original.get(*index as usize) {
                    Some(clause) => Ok(clause),
                    None => Err(err::ClauseDB::Missing),
                }
            }
            ClauseKey::Binary(index) => {
                //
                match self.binary.get(*index as usize) {
                    Some(clause) => Ok(clause),
                    None => Err(err::ClauseDB::Missing),
                }
            }
            ClauseKey::Addition(index, token) => {
                //
                match self.addition.get(*index as usize) {
                    Some(Some(clause)) => match clause.key() {
                        ClauseKey::Addition(_, clause_token) if &clause_token == token => {
                            Ok(clause)
                        }
                        _ => Err(err::ClauseDB::InvalidKeyToken),
                    },
                    Some(None) => Err(err::ClauseDB::InvalidKeyIndex),
                    None => Err(err::ClauseDB::InvalidKeyIndex),
                }
            }
        }
    }

    /// Returns Ok(mutable clause) corresponding to the given key, or an Err(issue) otherwise.
    ///
    /// ```rust, ignore
    /// self.clause_db.get_db_clause_mut(&key)?
    /// ```
    pub fn get_db_clause_mut(&mut self, key: &ClauseKey) -> Result<&mut dbClause, err::ClauseDB> {
        match key {
            ClauseKey::Unit(_) => Err(err::ClauseDB::GetUnitKey),
            ClauseKey::Original(index) => {
                //
                match self.original.get_mut(*index as usize) {
                    Some(clause) => Ok(clause),
                    None => Err(err::ClauseDB::Missing),
                }
            }
            ClauseKey::Binary(index) => {
                //
                match self.binary.get_mut(*index as usize) {
                    Some(clause) => Ok(clause),
                    None => Err(err::ClauseDB::Missing),
                }
            }
            ClauseKey::Addition(index, token) => {
                //
                match self.addition.get_mut(*index as usize) {
                    Some(Some(clause)) => match clause.key() {
                        ClauseKey::Addition(_, clause_token) if &clause_token == token => {
                            Ok(clause)
                        }
                        _ => Err(err::ClauseDB::InvalidKeyToken),
                    },
                    Some(None) => Err(err::ClauseDB::InvalidKeyIndex),
                    None => Err(err::ClauseDB::InvalidKeyIndex),
                }
            }
        }
    }

    /// Returns a result of the clause for a given key.
    ///
    /// No check is made on whether a clause is stored by the key.
    /// ```rust, ignore
    /// self.clause_db.get_db_clause_unchecked(&key)?
    /// ```
    /// # Safety
    /// To be used only when there is a guarantee that the clause has not been removed.
    ///
    /// E.g., this is safe to use with binary clauses, but not with addition clauses.
    pub unsafe fn get_db_clause_unchecked(
        &self,
        key: &ClauseKey,
    ) -> Result<&dbClause, err::ClauseDB> {
        match key {
            ClauseKey::Unit(_) => Err(err::ClauseDB::GetUnitKey),
            ClauseKey::Original(index) => Ok(self.original.get_unchecked(*index as usize)),
            ClauseKey::Binary(index) => Ok(self.binary.get_unchecked(*index as usize)),
            ClauseKey::Addition(index, token) => {
                //
                match self.addition.get_unchecked(*index as usize) {
                    Some(clause) => match clause.key() {
                        ClauseKey::Addition(_, clause_token) if &clause_token == token => {
                            Ok(clause)
                        }
                        _ => Err(err::ClauseDB::InvalidKeyToken),
                    },
                    None => Err(err::ClauseDB::InvalidKeyIndex),
                }
            }
        }
    }
}

impl ClauseDB {
    /// Notes the use of a clause.
    ///
    /// ```rust,ignore
    /// self.clause_db.note_use(key);
    /// ```
    /// In particular, the removal of an addition clause from the activity heap to prevent it's removal.
    pub fn note_use(&mut self, key: ClauseKey) {
        match key {
            ClauseKey::Addition(index, _) => {
                self.activity_heap.remove(index as usize);
            }
            ClauseKey::Unit(_) | ClauseKey::Binary(_) | ClauseKey::Original(_) => {}
        }
    }

    /// Places every addition clause on the activity heap and ensures the integrity of the heap.
    ///
    /// After this is called every addition clause is a candidate for deletion.
    pub fn refresh_heap(&mut self) {
        for (index, slot) in self.addition.iter().enumerate() {
            if slot.is_some() {
                self.activity_heap.activate(index);
            }
        }
        self.activity_heap.reheap();
    }

    /*
    To keep things simple a formula clause is ignored while a learnt clause is deleted
    */

    /// Removed addition clauses from the database up to the given limit (to remove) by taking keys from the activity heap.
    ///
    /// ```rust,ignore
    /// if self.scheduled_by_luby() {
    ///     self.clause_db.reduce_by(self.clause_db.current_addition_count() / 2);
    /// }
    /// ```
    // TODO: Improvements…?
    // For example, before dropping a clause the lbd could be recalculated…
    pub fn reduce_by(&mut self, limit: usize) -> Result<(), err::ClauseDB> {
        'reduction_loop: for _ in 0..limit {
            if let Some(index) = self.activity_heap.peek_max() {
                let value = self.activity_heap.value_at(index);
                log::debug!(target: targets::REDUCTION, "Took: {:?}", value);
                if value.lbd <= self.config.lbd_bound {
                    break 'reduction_loop;
                } else {
                    self.remove_addition(index)?;
                }
            } else {
                log::warn!(target: targets::REDUCTION, "Reduction called but there were no candidates");
            }
        }

        log::debug!(target: targets::REDUCTION, "Learnt clauses reduced to: {}", self.addition.len());
        Ok(())
    }

    /*
    Removing from learned checks to ensure removal is ok
    As the elements are optional for reuse, take places None at the index, as would be needed anyway
     */
    /// Removes an addition clause at the given index, and sends a dispatch if possible.
    fn remove_addition(&mut self, index: usize) -> Result<(), err::ClauseDB> {
        if unsafe { self.addition.get_unchecked(index) }.is_none() {
            log::error!(target: targets::CLAUSE_DB, "attempt to remove something that is not there");
            Err(err::ClauseDB::Missing)
        } else {
            // assert!(matches!(the_clause.key(), ClauseKey::LearnedLong(_, _)));
            let the_clause =
                std::mem::take(unsafe { self.addition.get_unchecked_mut(index) }).unwrap();

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
            self.addition_count -= 1;
            Ok(())
        }
    }

    /// Bumps the acitivty of a clause, rescoring all acitivies if needed.
    ///
    /// ```rust,ignore
    /// if let ClauseKey::Addition(index, _) = conflict {
    ///     clause_db.bump_activity(*index)
    /// };
    /// ```
    /// See the corresponding method with respect to atoms for more detials.
    pub fn bump_activity(&mut self, index: FormulaIndex) {
        if let Some(max) = self.activity_heap.peek_max_value() {
            if max.activity + self.config.bump > self.config.max_bump {
                let factor = 1.0 / max.activity;
                let decay_activity = |s: &ActivityLBD| ActivityLBD {
                    activity: s.activity * factor,
                    lbd: s.lbd,
                };
                self.activity_heap.apply_to_all(decay_activity);
                self.config.bump *= factor
            }
        }

        let bump_activity = |s: &ActivityLBD| ActivityLBD {
            activity: s.activity + self.config.bump,
            lbd: s.lbd,
        };

        let index = index as usize;
        self.activity_heap.apply_to_index(index, bump_activity);
        self.activity_heap.heapify_if_active(index);

        self.config.bump *= 1.0 / (1.0 - self.config.decay);
    }

    /// The count of all clauses encountered, including removed clauses.
    pub fn total_clause_count(&self) -> usize {
        self.unit.len() + self.original.len() + self.binary.len() + self.addition_count
    }

    /// The count of all clauses currently in the context.
    pub fn current_clause_count(&self) -> usize {
        self.unit.len() + self.original.len() + self.binary.len() + self.addition.len()
    }

    /// The count of the current addition clauses in the context.
    ///
    /// ```rust,ignore
    /// self.clause_db.reduce_by(self.clause_db.current_addition_count() / 2);
    /// ```
    pub fn current_addition_count(&self) -> usize {
        self.addition.len()
    }

    /// An iterator over all unit clauses, given as [abLiteral]s.
    ///
    /// ```rust,ignore
    /// buffer.strengthen_given(self.clause_db.all_unit_clauses());
    /// ```
    pub fn all_unit_clauses(&self) -> impl Iterator<Item = &abLiteral> {
        self.unit.iter()
    }

    /// An iterator over all non-unit clauses, given as [impl Clause]s.
    ///
    /// ```rust,ignore
    /// let mut clause_iter = the_context.clause_db.all_nonunit_clauses();
    /// ```
    pub fn all_nonunit_clauses(&self) -> impl Iterator<Item = &impl Clause> + '_ {
        self.original.iter().chain(
            self.binary.iter().chain(
                self.addition
                    .iter()
                    .flat_map(|maybe_clause| maybe_clause.as_ref()),
            ),
        )
    }

    /// Send a dispatch of all active clauses.
    pub fn dispatch_active(&self) {
        if let Some(dispatcher) = &self.dispatcher {
            for literal in self.all_unit_clauses() {
                let report = report::ClauseDB::ActiveUnit(*literal);
                dispatcher(Dispatch::Report(report::Report::ClauseDB(report)));
            }

            for clause in &self.binary {
                let report = report::ClauseDB::Active(clause.key(), clause.to_vec());
                dispatcher(Dispatch::Report(Report::ClauseDB(report)));
            }

            for clause in &self.original {
                let report = report::ClauseDB::Active(clause.key(), clause.to_vec());
                dispatcher(Dispatch::Report(Report::ClauseDB(report)));
            }

            for clause in self.addition.iter().flatten() {
                if clause.is_active() {
                    let report = report::ClauseDB::Active(clause.key(), clause.to_vec());
                    dispatcher(Dispatch::Report(Report::ClauseDB(report)));
                }
            }
        }
    }

    /// Removed the given literal from the clause indexed by the given key, if possible.
    ///
    /// As the clause may become binary, the returned key should be used.
    ///
    /// ```rust, ignore
    /// let new_key = clause_db.subsume(old_key, literal, atom_db)?;
    /// ```
    ///
    /// At present, this is limited to clauses with three or more literals.
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
        literal: impl Borrow<abLiteral>,
        atom_db: &mut AtomDB,
    ) -> Result<ClauseKey, err::Subsumption> {
        let the_clause = self.get_db_clause_mut(&key).unwrap();
        match the_clause.len() {
            0..=2 => Err(err::Subsumption::ClauseTooShort),
            3 => {
                the_clause.subsume(literal, atom_db, false)?;
                let Ok(new_key) = self.transfer_to_binary(key, atom_db) else {
                    return Err(err::Subsumption::TransferFailure);
                };
                Ok(new_key)
            }
            _ => {
                the_clause.subsume(literal, atom_db, true)?;
                // TODO: Dispatches for subsumption…
                // let delta = delta::Resolution::Subsumed(key, literal);
                // (Dispatch::Resolution(delta));
                Ok(key)
            }
        }
    }
}
