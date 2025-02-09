//! A database of clause related things.
//!
//! Records of clauses are distinguished by a mix of [kind](crate::structures::clause::ClauseKind) and/or [source](crate::structures::clause::ClauseSource).
//!
//! Fields of the database are private to ensure the use of methods which may be needed to uphold invariants.
pub mod activity_glue;
pub mod callbacks;
pub mod db_clause;
mod get;
mod store;
mod transfer;

use std::{
    borrow::Borrow,
    collections::{HashMap, HashSet},
    rc::Rc,
};

use callbacks::{CallbackAddition, CallbackDelete, CallbackFixed};
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
        macros::{self},
        Dispatch,
    },
    generic::index_heap::IndexHeap,
    misc::log::targets::{self},
    structures::{
        clause::{CClause, Clause},
        literal::CLiteral,
    },
    types::err::{self},
};

/// A database of clause related things.
pub struct ClauseDB {
    /// Clause database specific configuration parameters.
    config: ClauseDBConfig,

    /// A count of addition clauses.
    // This can't be inferred from the addition vec, as indicies may be reused.
    addition_count: usize,

    /// A stack of keys for learned clauses whose indicies are empty.
    empty_keys: Vec<ClauseKey>,

    /// Original unit clauses.
    unit_original: HashMap<ClauseKey, dbClause>,

    /// Additionl unit clauses.
    unit_addition: HashMap<ClauseKey, dbClause>,

    /// Binary clauses.
    binary_original: Vec<dbClause>,

    /// Binary clauses.
    binary_addition: Vec<dbClause>,

    /// Original clauses.
    original: Vec<dbClause>,

    /// Addition clauses.
    addition: Vec<Option<dbClause>>,

    /// An activity heap of addition clause keys.
    activity_heap: IndexHeap<ActivityLBD>,

    /// Where to send dispatches.
    dispatcher: Option<Rc<dyn Fn(Dispatch)>>,

    /// Addition clauses are passed in.
    pub(super) callback_addition: Option<Box<CallbackAddition>>,

    /// Deleted clauses are passed in.
    pub(super) callback_delete: Option<Box<CallbackDelete>>,

    /// Fixed literals are passed in.
    pub(super) callback_fixed: Option<Box<CallbackFixed>>,
}

impl ClauseDB {
    pub fn new(config: &Config, dispatcher: Option<Rc<dyn Fn(Dispatch)>>) -> Self {
        ClauseDB {
            addition_count: 0,
            empty_keys: Vec::default(),

            unit_original: HashMap::default(),
            unit_addition: HashMap::default(),

            binary_original: Vec::default(),
            binary_addition: Vec::default(),

            original: Vec::default(),
            addition: Vec::default(),

            activity_heap: IndexHeap::default(),
            config: config.clause_db.clone(),

            dispatcher,

            callback_addition: None,
            callback_delete: None,
            callback_fixed: None,
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
            ClauseKey::OriginalUnit(_)
            | ClauseKey::AdditionUnit(_)
            | ClauseKey::OriginalBinary(_)
            | ClauseKey::AdditionBinary(_)
            | ClauseKey::Original(_) => {}
        }
    }

    /// Places every addition clause on the activity heap and ensures the integrity of the heap.
    ///
    /// After this is called every addition clause is a candidate for deletion.
    pub fn refresh_heap(&mut self) {
        for (index, slot) in self.addition.iter().enumerate() {
            if let Some(clause) = slot {
                if clause.proof_occurrence_count() == 0 {
                    self.activity_heap.activate(index);
                }
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
    pub fn reduce_by(&mut self, limit: usize) -> Result<(), err::ClauseDBError> {
        'reduction_loop: for _ in 0..limit {
            if let Some(index) = self.activity_heap.peek_max() {
                let value = self.activity_heap.value_at(index);
                log::debug!(target: targets::REDUCTION, "Took ~ Activity: {} LBD: {}", value.activity, value.lbd);

                if value.lbd <= self.config.lbd_bound {
                    break 'reduction_loop;
                } else {
                    self.remove_addition(index)?;
                }
            } else {
                log::warn!(target: targets::REDUCTION, "Reduction called but there were no candidates");
            }
        }

        log::info!(target: targets::REDUCTION, "Addition clauses reduced to: {}", self.addition.len());
        Ok(())
    }

    /*
    Removing from learned checks to ensure removal is ok
    As the elements are optional for reuse, take places None at the index, as would be needed anyway
     */
    /// Removes an addition clause at the given index, and sends a dispatch if possible.
    fn remove_addition(&mut self, index: usize) -> Result<(), err::ClauseDBError> {
        let the_clause = std::mem::take(unsafe { self.addition.get_unchecked_mut(index) });
        match the_clause {
            None => {
                log::error!(target: targets::CLAUSE_DB, "Remove called on a missing addition clause");
                Err(err::ClauseDBError::Missing)
            }
            Some(the_clause) => {
                self.make_callback_delete(the_clause.clause());

                for premise_key in the_clause.premises() {
                    match premise_key {
                        ClauseKey::Addition(_, _) => {
                            let clause = unsafe { self.get_unchecked_mut(premise_key).unwrap() };
                            clause.decrement_proof_count();
                            if !clause.is_active() && clause.proof_occurrence_count() == 0 {
                                self.activity_heap.activate(premise_key.index());
                            }
                        }

                        _ => {}
                    }
                }

                macros::dispatch_clause_removal!(self, the_clause);

                self.activity_heap.remove(index);
                self.empty_keys.push(*the_clause.key());
                self.addition_count -= 1;
                Ok(())
            }
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
        self.unit_original.len()
            + self.unit_addition.len()
            + self.original.len()
            + self.binary_original.len()
            + self.binary_addition.len()
            + self.addition_count
    }

    /// The count of all clauses currently in the context.
    pub fn current_clause_count(&self) -> usize {
        self.unit_original.len()
            + self.unit_addition.len()
            + self.original.len()
            + self.binary_original.len()
            + self.binary_addition.len()
            + self.addition.len()
    }

    /// The count of the current addition clauses in the context.
    ///
    /// ```rust,ignore
    /// self.clause_db.reduce_by(self.clause_db.current_addition_count() / 2);
    /// ```
    pub fn current_addition_count(&self) -> usize {
        self.addition.len()
    }

    /// An iterator over all original unit clauses, given as [CLiteral]s.
    pub fn all_original_unit_clauses(
        &self,
    ) -> impl Iterator<Item = (ClauseKey, CLiteral)> + use<'_> {
        self.unit_original.values().flat_map(|c| {
            c.clause()
                .literals()
                .last()
                .map(|literal| (ClauseKey::OriginalUnit(literal), literal))
        })
    }

    /// An iterator over all addition unit clauses, given as [CLiteral]s.
    pub fn all_addition_unit_clauses(
        &self,
    ) -> impl Iterator<Item = (ClauseKey, CLiteral)> + use<'_> {
        self.unit_addition.values().flat_map(|c| {
            c.clause()
                .literals()
                .last()
                .map(|literal| (ClauseKey::AdditionUnit(literal), literal))
        })
    }

    /// An iterator over all unit clauses, given as [CLiteral]s.
    ///
    /// ```rust,ignore
    /// buffer.strengthen_given(self.clause_db.all_unit_clauses());
    /// ```
    pub fn all_unit_clauses(&self) -> impl Iterator<Item = (ClauseKey, CLiteral)> + use<'_> {
        self.all_original_unit_clauses()
            .chain(self.all_addition_unit_clauses())
    }

    /// An iterator over all original binary clauses.
    pub fn all_original_binary_clauses(
        &self,
    ) -> impl Iterator<Item = (ClauseKey, &CClause)> + use<'_> {
        self.binary_original.iter().map(|c| (*c.key(), c.clause()))
    }

    /// An iterator over all addition binary clauses.
    pub fn all_addition_binary_clauses(
        &self,
    ) -> impl Iterator<Item = (ClauseKey, &CClause)> + use<'_> {
        self.binary_addition.iter().map(|c| (*c.key(), c.clause()))
    }

    /// An iterator over all addition binary clauses.
    pub fn all_binary_clauses(&self) -> impl Iterator<Item = (ClauseKey, &CClause)> + use<'_> {
        self.all_original_binary_clauses()
            .chain(self.all_addition_binary_clauses())
    }

    /// An iterator over all original binary clauses.
    pub fn all_original_long_clauses(
        &self,
    ) -> impl Iterator<Item = (ClauseKey, &CClause)> + use<'_> {
        self.original.iter().map(|c| (*c.key(), c.clause()))
    }

    /// An iterator over all addition binary clauses.
    pub fn all_addition_long_clauses(
        &self,
    ) -> impl Iterator<Item = (ClauseKey, &CClause)> + use<'_> {
        self.addition
            .iter()
            .flat_map(|c| c.as_ref().map(|db_c| (*db_c.key(), db_c.clause())))
    }

    /// An iterator over all addition binary clauses.
    pub fn all_long_clauses(&self) -> impl Iterator<Item = (ClauseKey, &CClause)> + use<'_> {
        self.all_original_long_clauses()
            .chain(self.all_addition_long_clauses())
    }

    /// An iterator over all non-unit clauses, given as [impl Clause]s.
    ///
    /// ```rust,ignore
    /// let mut clause_iter = the_context.clause_db.all_nonunit_clauses();
    /// ```
    pub fn all_nonunit_clauses(&self) -> impl Iterator<Item = (ClauseKey, &CClause)> + use<'_> {
        self.all_binary_clauses().chain(self.all_long_clauses())
    }

    /// Send a dispatch of all active clauses.
    pub fn dispatch_active(&self) {
        if let Some(dispatcher) = &self.dispatcher {
            for (_, literal) in self.all_original_unit_clauses() {
                let report = report::ClauseDBReport::ActiveOriginalUnit(literal);
                dispatcher(Dispatch::Report(report::Report::ClauseDB(report)));
            }

            for (_, literal) in self.all_addition_unit_clauses() {
                let report = report::ClauseDBReport::ActiveAdditionUnit(literal);
                dispatcher(Dispatch::Report(report::Report::ClauseDB(report)));
            }

            for clause in &self.binary_original {
                let report = report::ClauseDBReport::Active(*clause.key(), clause.to_vec());
                dispatcher(Dispatch::Report(Report::ClauseDB(report)));
            }

            for clause in &self.binary_addition {
                let report = report::ClauseDBReport::Active(*clause.key(), clause.to_vec());
                dispatcher(Dispatch::Report(Report::ClauseDB(report)));
            }

            for clause in &self.original {
                let report = report::ClauseDBReport::Active(*clause.key(), clause.to_vec());
                dispatcher(Dispatch::Report(Report::ClauseDB(report)));
            }

            for clause in self.addition.iter().flatten() {
                if clause.is_active() {
                    let report = report::ClauseDBReport::Active(*clause.key(), clause.to_vec());
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
    ///
    /// # Safety
    /// Assumes a clause is indexed by the key.
    /*
    If the resolved clause is binary then subsumption transfers the clause to the store for binary clauses
    This is safe to do as:
    - After backjumping all the observations at the current level will be forgotten
    - The clause does not appear in the observations of any previous stage
      + As, if the clause appeared in some previous stage then use of the clause would be a repeat implication
      + And, repeat implications are checked prior to conflicts
     */
    pub unsafe fn subsume(
        &mut self,
        key: ClauseKey,
        literal: impl Borrow<CLiteral>,
        atom_db: &mut AtomDB,
        premises: HashSet<ClauseKey>,
    ) -> Result<ClauseKey, err::SubsumptionError> {
        let the_clause = match self.get_unchecked_mut(&key) {
            Ok(c) => c,
            Err(_) => return Err(err::SubsumptionError::ClauseDB),
        };
        match the_clause.len() {
            0..=2 => Err(err::SubsumptionError::ClauseTooShort),
            3 => {
                the_clause.subsume(literal, atom_db, false)?;
                let Ok(new_key) = self.transfer_to_binary(key, atom_db, premises) else {
                    return Err(err::SubsumptionError::TransferFailure);
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
