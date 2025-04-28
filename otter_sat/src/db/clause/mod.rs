/*!
A database of clause related things.

Records of clauses are distinguished by a mix of [kind](crate::structures::clause::ClauseKind) and/or [source](crate::structures::clause::ClauseSource).

Fields of the database are private to ensure the use of methods which may be needed to uphold invariants.
*/
pub mod activity_glue;
mod callbacks;
pub mod db_clause;
mod get;
mod iterators;
mod store;

use std::collections::HashMap;

use db_clause::DBClause;

use crate::{
    config::{Config, dbs::ClauseDBConfig},
    context::callbacks::{CallbackOnClause, CallbackOnClauseSource, CallbackOnLiteral},
    db::{clause::activity_glue::ActivityLBD, keys::ClauseKey},
    generic::index_heap::IndexHeap,
    misc::log::targets::{self},
    types::err::{self},
};

/// A database of clause related things.
pub struct ClauseDB {
    /// Clause database specific configuration parameters.
    pub(super) config: ClauseDBConfig,

    /// A count of addition clauses.
    // This can't be inferred from the addition vec, as indices may be reused.
    addition_count: usize,

    /// A stack of keys for learned clauses whose indices are empty.
    empty_keys: Vec<ClauseKey>,

    /// Original unit clauses.
    unit_original: HashMap<ClauseKey, DBClause>,

    /// Additionl unit clauses.
    unit_addition: HashMap<ClauseKey, DBClause>,

    /// Binary clauses.
    binary_original: Vec<DBClause>,

    /// Binary clauses.
    binary_addition: Vec<DBClause>,

    /// Original clauses.
    original: Vec<DBClause>,

    /// Addition clauses.
    addition: Vec<Option<DBClause>>,

    /// An activity heap of addition clause keys.
    pub(super) activity_heap: IndexHeap<ActivityLBD>,

    /// Resolution graph
    pub resolution_graph: HashMap<ClauseKey, Vec<ClauseKey>>,

    /// Original clauses are passed in.
    callback_original: Option<Box<CallbackOnClauseSource>>,

    /// Addition clauses are passed in.
    callback_addition: Option<Box<CallbackOnClauseSource>>,

    /// Deleted clauses are passed in.
    callback_delete: Option<Box<CallbackOnClause>>,

    /// Fixed literals are passed in.
    callback_fixed: Option<Box<CallbackOnLiteral>>,

    /// The unsatisfiable clause is passed in.
    callback_unsatisfiable: Option<Box<CallbackOnClause>>,
}

impl ClauseDB {
    /// A new [ClauseDB] with local configuration options derived from `config`.
    pub fn new(config: &Config) -> Self {
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
            resolution_graph: HashMap::default(),

            config: config.clause_db.clone(),

            callback_original: None,
            callback_addition: None,
            callback_delete: None,
            callback_fixed: None,
            callback_unsatisfiable: None,
        }
    }
}

impl ClauseDB {
    /// Locks an addition clause, preventing the removal of addition clauses until unlocked.
    /// Returns true if a lock was placed and false otherwise.
    pub fn lock_addition_clause(&mut self, key: ClauseKey) -> bool {
        match key {
            ClauseKey::Addition(index, _) => self.activity_heap.remove(index as usize),
            ClauseKey::OriginalUnit(_)
            | ClauseKey::AdditionUnit(_)
            | ClauseKey::OriginalBinary(_)
            | ClauseKey::AdditionBinary(_)
            | ClauseKey::Original(_) => false,
        }
    }

    /// Make every addition clause a clause a candidate for deletion by placing each on the activity heap.
    pub fn unlock_all_addition_clauses(&mut self) {
        for (index, clause_slot) in self.addition.iter().enumerate() {
            if clause_slot.is_some() {
                self.activity_heap.activate(index);
            }
        }
        self.activity_heap.heapify();
    }

    /// Removed addition clauses from the database up to the given limit (to remove) by taking keys from the activity heap.
    // TODO: Improvements…?
    // For example, before dropping a clause the lbd could be recalculated…
    pub fn reduce_by(&mut self, limit: usize) -> Result<(), err::ClauseDBError> {
        'reduction_loop: for _ in 0..limit {
            if let Some(index) = self.activity_heap.peek_max() {
                let value = self.activity_heap.value_at(index);
                log::debug!(target: targets::REDUCTION, "Took ~ Activity: {} LBD: {}", value.activity, value.lbd);

                if value.lbd <= self.config.lbd_bound.value {
                    break 'reduction_loop;
                } else {
                    // # Safety: Index is drawn from the activity heap, which matches the size of the addition db.
                    unsafe { self.remove_addition(index) }?;
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
    ///
    /// # Safety
    /// The clause db size for additions must exceed `index`.
    /// Though, does not require there is a clause stored at `index`.
    unsafe fn remove_addition(&mut self, index: usize) -> Result<(), err::ClauseDBError> {
        // # Safety: By assumption, the clause db size for additions exceeds `index`.
        let to_remove = std::mem::take(unsafe { self.addition.get_unchecked_mut(index) });

        match to_remove {
            None => {
                log::error!(target: targets::CLAUSE_DB, "Remove called on a missing addition clause");
                Err(err::ClauseDBError::Missing)
            }
            Some(clause) => {
                self.make_callback_delete(&clause);

                self.activity_heap.remove(index);
                self.empty_keys.push(*clause.key());
                self.addition_count -= 1;
                Ok(())
            }
        }
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
    pub fn current_addition_count(&self) -> usize {
        self.addition.len()
    }
}
