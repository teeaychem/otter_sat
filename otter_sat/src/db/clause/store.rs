use std::collections::HashSet;

use crate::{
    db::{
        atom::AtomDB,
        clause::{activity_glue::ActivityLBD, db_clause::dbClause},
        ClauseKey, FormulaIndex,
    },
    dispatch::{
        library::delta::{self, Delta},
        macros::{self},
        Dispatch,
    },
    ipasir::IpasirCallbacks,
    misc::log::targets,
    structures::{
        clause::{CClause, Clause, ClauseSource},
        literal::CLiteral,
        valuation::vValuation,
    },
    types::err,
};

use super::ClauseDB;

/// Methods for storing clauses.
///
/// As key generation is local, the methods are not exported.
/// Though, note, as keys use a [index](FormulaIndex) which may be smaller than [usize] a check is made to ensure it will be possible to generate the key.
impl ClauseDB {
    /// Stores a clause with an automatically generated id.
    ///
    /// ```rust, ignore
    /// self.clause_db.store(clause, ClauseSource::Resolution, &mut self.atom_db, None);
    /// ```
    ///
    /// Any instance of storing a clause should directly or indirectly use this method, as it maintains the activity heap, watched literals, etc.
    /// A valuation is optional.
    /// If given, clauses are initialised with respect to the given valuation.
    /// Otherwise, clauses are initialised with respect to the current valuation of the context.
    pub fn store(
        &mut self,
        clause: impl Clause,
        source: ClauseSource,
        atom_db: &mut AtomDB,
        valuation: Option<&vValuation>,
        premises: HashSet<ClauseKey>,
        callbacks: &Option<IpasirCallbacks>,
    ) -> Result<ClauseKey, err::ClauseDBError> {
        match clause.size() {
            0 => Err(err::ClauseDBError::EmptyClause),

            // The match ensures there is a next (and then no further) literal in the clause.
            1 => {
                let literal = unsafe { clause.literals().next().unwrap_unchecked() };
                self.store_unit(literal, source, premises, callbacks)
            }

            2 => self.store_binary(
                clause.canonical(),
                source,
                atom_db,
                valuation,
                premises,
                callbacks,
            ),

            _ => self.store_long(
                clause.canonical(),
                source,
                atom_db,
                valuation,
                premises,
                callbacks,
            ),
        }
    }
}

impl ClauseDB {
    fn store_unit(
        &mut self,
        literal: CLiteral,
        source: ClauseSource,
        premises: HashSet<ClauseKey>,
        callbacks: &Option<IpasirCallbacks>,
    ) -> Result<ClauseKey, err::ClauseDBError> {
        match source {
            ClauseSource::Original => {
                let key = ClauseKey::OriginalUnit(literal);
                let clause = dbClause::new_unit(key, literal, premises);
                self.unit_original.insert(key, clause);

                macros::dispatch_clause_db_delta!(self, Original, key);

                Ok(key)
            }

            ClauseSource::BCP => {
                let key = ClauseKey::AdditionUnit(literal);
                let clause = dbClause::new_unit(key, literal, premises);
                self.unit_addition.insert(key, clause);

                macros::dispatch_clause_db_delta!(self, BCP, key);
                if let Some(callbacks) = callbacks {
                    unsafe { callbacks.call_ipasir_addition_callback(&vec![literal]) };
                }

                Ok(key)
            }

            ClauseSource::Resolution => {
                let key = ClauseKey::AdditionUnit(literal);
                let clause = dbClause::new_unit(key, literal, premises);
                self.unit_addition.insert(key, clause);

                macros::dispatch_clause_db_delta!(self, Added, key);
                if let Some(callbacks) = callbacks {
                    unsafe { callbacks.call_ipasir_addition_callback(&vec![literal]) };
                }

                Ok(key)
            }

            ClauseSource::PureUnit => panic!("!"),
        }
    }

    fn store_binary(
        &mut self,
        clause: CClause,
        source: ClauseSource,
        atom_db: &mut AtomDB,
        valuation: Option<&vValuation>,
        premises: HashSet<ClauseKey>,
        callbacks: &Option<IpasirCallbacks>,
    ) -> Result<ClauseKey, err::ClauseDBError> {
        match source {
            ClauseSource::Original => {
                let key = self.fresh_original_binary_key()?;

                macros::dispatch_clause_addition!(self, clause, Original, key);

                let clause = dbClause::new_nonunit(key, clause, atom_db, valuation, premises);
                self.binary_original.push(clause);

                Ok(key)
            }

            ClauseSource::BCP | ClauseSource::Resolution => {
                let key = self.fresh_addition_binary_key()?;

                macros::dispatch_clause_addition!(self, clause, Added, key);
                if let Some(callbacks) = callbacks {
                    unsafe { callbacks.call_ipasir_addition_callback(&clause) };
                }

                let clause = dbClause::new_nonunit(key, clause, atom_db, valuation, premises);
                self.binary_addition.push(clause);

                Ok(key)
            }

            ClauseSource::PureUnit => panic!("!"),
        }
    }

    fn store_long(
        &mut self,
        clause: CClause,
        source: ClauseSource,
        atom_db: &mut AtomDB,
        valuation: Option<&vValuation>,
        premises: HashSet<ClauseKey>,
        callbacks: &Option<IpasirCallbacks>,
    ) -> Result<ClauseKey, err::ClauseDBError> {
        match source {
            ClauseSource::BCP | ClauseSource::PureUnit => {
                panic!("!")
            } // Sources only valid for unit clauses.

            ClauseSource::Original => {
                let key = self.fresh_original_key()?;

                macros::dispatch_clause_addition!(self, clause, Original, key);
                log::trace!(target: targets::CLAUSE_DB, "{key}: {}", clause.as_dimacs(false));

                let db_clause = dbClause::new_nonunit(key, clause, atom_db, valuation, premises);

                self.original.push(db_clause);
                Ok(key)
            }

            ClauseSource::Resolution => {
                self.addition_count += 1;

                let key = match self.empty_keys.len() {
                    0 => self.fresh_addition_key()?,
                    _ => self.empty_keys.pop().unwrap().retoken()?,
                };

                macros::dispatch_clause_addition!(self, clause, Added, key);
                if let Some(callbacks) = callbacks {
                    unsafe { callbacks.call_ipasir_addition_callback(&clause) };
                }

                log::trace!(target: targets::CLAUSE_DB, "{key}: {}", clause.as_dimacs(false));

                let stored_form = dbClause::new_nonunit(key, clause, atom_db, valuation, premises);

                let value = ActivityLBD {
                    activity: 1.0,
                    lbd: stored_form.lbd(atom_db),
                };

                self.activity_heap.add(key.index(), value);
                self.activity_heap.activate(key.index());

                match key {
                    ClauseKey::Addition(_, 0) => self.addition.push(Some(stored_form)),

                    ClauseKey::Addition(_, _) => unsafe {
                        *self.addition.get_unchecked_mut(key.index()) = Some(stored_form)
                    },

                    _ => panic!("!"),
                };

                Ok(key)
            }
        }
    }
}

impl ClauseDB {
    /// A key to a binary clause.
    pub(super) fn fresh_original_binary_key(&mut self) -> Result<ClauseKey, err::ClauseDBError> {
        if self.binary_original.len() == FormulaIndex::MAX as usize {
            return Err(err::ClauseDBError::StorageExhausted);
        }
        let key = ClauseKey::OriginalBinary(self.binary_original.len() as FormulaIndex);
        Ok(key)
    }

    pub(super) fn fresh_addition_binary_key(&mut self) -> Result<ClauseKey, err::ClauseDBError> {
        if self.binary_addition.len() == FormulaIndex::MAX as usize {
            return Err(err::ClauseDBError::StorageExhausted);
        }
        let key = ClauseKey::AdditionBinary(self.binary_addition.len() as FormulaIndex);
        Ok(key)
    }

    /// A key to an original clause.
    fn fresh_original_key(&mut self) -> Result<ClauseKey, err::ClauseDBError> {
        if self.original.len() == FormulaIndex::MAX as usize {
            return Err(err::ClauseDBError::StorageExhausted);
        }
        let key = ClauseKey::Original(self.original.len() as FormulaIndex);
        Ok(key)
    }

    /// A key to an addition clause.
    fn fresh_addition_key(&mut self) -> Result<ClauseKey, err::ClauseDBError> {
        if self.addition.len() == FormulaIndex::MAX as usize {
            return Err(err::ClauseDBError::StorageExhausted);
        }
        let key = ClauseKey::Addition(self.addition.len() as FormulaIndex, 0);
        Ok(key)
    }
}
