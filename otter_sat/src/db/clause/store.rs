use std::collections::HashSet;

use crate::{
    db::{
        ClauseKey, FormulaIndex,
        atom::AtomDB,
        clause::{activity_glue::ActivityLBD, db_clause::dbClause},
    },
    misc::log::targets,
    structures::{
        clause::{CClause, Clause, ClauseSource},
        literal::CLiteral,
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
    /// Any instance of storing a clause should directly or indirectly use this method, as it maintains the activity heap, watched literals, etc.
    /// A valuation is optional.
    /// If given, clauses are initialised with respect to the given valuation.
    /// Otherwise, clauses are initialised with respect to the current valuation of the context.
    pub fn store(
        &mut self,
        clause: impl Clause,
        source: ClauseSource,
        atom_db: &mut AtomDB,
        premises: HashSet<ClauseKey>,
    ) -> Result<ClauseKey, err::ClauseDBError> {
        let key = match clause.size() {
            0 => Err(err::ClauseDBError::EmptyClause),

            // The match ensures there is a next (and then no further) literal in the clause.
            1 => {
                let literal = unsafe { clause.literals().next().unwrap_unchecked() };
                self.store_unit(literal, source)
            }

            2 => self.store_binary(clause.canonical(), source, atom_db),

            _ => self.store_long(clause.canonical(), source, atom_db),
        };

        if let Ok(key) = key {
            self.resolution_graph
                .insert(key, premises.into_iter().collect());
        }

        key
    }
}

impl ClauseDB {
    fn store_unit(
        &mut self,
        literal: CLiteral,
        source: ClauseSource,
    ) -> Result<ClauseKey, err::ClauseDBError> {
        match source {
            ClauseSource::Original => {
                let key = ClauseKey::OriginalUnit(literal);
                let clause = dbClause::new_unit(key, literal);

                self.make_callback_original(&clause, &source);
                self.unit_original.insert(key, clause);

                Ok(key)
            }

            ClauseSource::BCP => {
                let key = ClauseKey::AdditionUnit(literal);
                let clause = dbClause::new_unit(key, literal);

                self.make_callback_addition(&clause, &source);
                self.unit_addition.insert(key, clause);

                self.make_callback_fixed(literal);

                Ok(key)
            }

            ClauseSource::Resolution => {
                let key = ClauseKey::AdditionUnit(literal);
                let clause = dbClause::new_unit(key, literal);
                self.make_callback_addition(&clause, &source);
                self.unit_addition.insert(key, clause);

                self.make_callback_fixed(literal);

                Ok(key)
            }

            ClauseSource::PureUnit => panic!("! Store of a pure unit clause"),
        }
    }

    fn store_binary(
        &mut self,
        clause: CClause,
        source: ClauseSource,
        atom_db: &mut AtomDB,
    ) -> Result<ClauseKey, err::ClauseDBError> {
        match source {
            ClauseSource::Original => {
                let key = self.fresh_original_binary_key()?;
                let clause = dbClause::new_nonunit(key, clause, atom_db);

                self.make_callback_original(&clause, &source);
                self.binary_original.push(clause);

                Ok(key)
            }

            ClauseSource::BCP | ClauseSource::Resolution => {
                let key = self.fresh_addition_binary_key()?;
                let clause = dbClause::new_nonunit(key, clause, atom_db);

                self.make_callback_addition(&clause, &source);
                self.binary_addition.push(clause);

                Ok(key)
            }

            ClauseSource::PureUnit => panic!("! Store of a pure unit clause"),
        }
    }

    fn store_long(
        &mut self,
        clause: CClause,
        source: ClauseSource,
        atom_db: &mut AtomDB,
    ) -> Result<ClauseKey, err::ClauseDBError> {
        match source {
            ClauseSource::BCP | ClauseSource::PureUnit => {
                panic!("! Store of non-long clause via store_long")
            } // Sources only valid for unit clauses.

            ClauseSource::Original => {
                let key = self.fresh_original_key()?;

                log::trace!(target: targets::CLAUSE_DB, "{key}: {}", clause.as_dimacs(false));

                let db_clause = dbClause::new_nonunit(key, clause, atom_db);
                self.make_callback_original(&db_clause, &source);

                self.original.push(db_clause);
                Ok(key)
            }

            ClauseSource::Resolution => {
                self.addition_count += 1;

                let key = match self.empty_keys.pop() {
                    None => self.fresh_addition_key()?,
                    Some(key) => key.retoken()?,
                };

                log::trace!(target: targets::CLAUSE_DB, "{key}: {}", clause.as_dimacs(false));

                let stored_form = dbClause::new_nonunit(key, clause, atom_db);
                self.make_callback_addition(&stored_form, &source);

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

                    _ => panic!("! Addition key required"),
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
