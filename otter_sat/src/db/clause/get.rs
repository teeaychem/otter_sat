use crate::{db::ClauseKey, types::err};

use super::{db_clause::dbClause, ClauseDB};

/// Methods to get clauses stored in the database.
impl ClauseDB {
    /// Returns Ok(clause) corresponding to the given key, or an Err(issue) otherwise.
    ///
    /// ```rust, ignore
    /// self.clause_db.get_db_clause(&key)?
    /// ```
    pub fn get(&self, key: &ClauseKey) -> Result<&dbClause, err::ClauseDBError> {
        match key {
            ClauseKey::OriginalUnit(_) => Err(err::ClauseDBError::GetOriginalUnitKey),

            ClauseKey::AdditionUnit(_) => {
                //
                match self.unit_addition.get(key) {
                    Some(clause) => Ok(clause),
                    None => Err(err::ClauseDBError::Missing),
                }
            }

            ClauseKey::Original(index) => {
                //
                match self.original.get(*index as usize) {
                    Some(clause) => Ok(clause),
                    None => Err(err::ClauseDBError::Missing),
                }
            }

            ClauseKey::OriginalBinary(index) => {
                //
                match self.binary_original.get(*index as usize) {
                    Some(clause) => Ok(clause),
                    None => Err(err::ClauseDBError::Missing),
                }
            }

            ClauseKey::AdditionBinary(index) => {
                //
                match self.binary_addition.get(*index as usize) {
                    Some(clause) => Ok(clause),
                    None => Err(err::ClauseDBError::Missing),
                }
            }

            ClauseKey::Addition(index, token) => {
                //
                match self.addition.get(*index as usize) {
                    Some(Some(clause)) => match clause.key() {
                        ClauseKey::Addition(_, clause_token) if clause_token == token => Ok(clause),
                        _ => Err(err::ClauseDBError::InvalidKeyToken),
                    },
                    Some(None) => Err(err::ClauseDBError::InvalidKeyIndex),
                    None => Err(err::ClauseDBError::InvalidKeyIndex),
                }
            }
        }
    }

    /// Returns Ok(mutable clause) corresponding to the given key, or an Err(issue) otherwise.
    ///
    /// ```rust, ignore
    /// self.clause_db.get_db_clause_mut(&key)?
    /// ```
    pub fn get_mut(&mut self, key: &ClauseKey) -> Result<&mut dbClause, err::ClauseDBError> {
        match key {
            ClauseKey::OriginalUnit(_) => Err(err::ClauseDBError::GetOriginalUnitKey),

            ClauseKey::AdditionUnit(_) => {
                //
                match self.unit_addition.get_mut(key) {
                    Some(clause) => Ok(clause),
                    None => Err(err::ClauseDBError::Missing),
                }
            }

            ClauseKey::Original(index) => {
                //
                match self.original.get_mut(*index as usize) {
                    Some(clause) => Ok(clause),
                    None => Err(err::ClauseDBError::Missing),
                }
            }

            ClauseKey::OriginalBinary(index) => {
                //
                match self.binary_original.get_mut(*index as usize) {
                    Some(clause) => Ok(clause),
                    None => Err(err::ClauseDBError::Missing),
                }
            }

            ClauseKey::AdditionBinary(index) => {
                //
                match self.binary_addition.get_mut(*index as usize) {
                    Some(clause) => Ok(clause),
                    None => Err(err::ClauseDBError::Missing),
                }
            }

            ClauseKey::Addition(index, token) => {
                //
                match self.addition.get_mut(*index as usize) {
                    Some(Some(clause)) => match clause.key() {
                        ClauseKey::Addition(_, clause_token) if clause_token == token => Ok(clause),
                        _ => Err(err::ClauseDBError::InvalidKeyToken),
                    },
                    Some(None) => Err(err::ClauseDBError::InvalidKeyIndex),
                    None => Err(err::ClauseDBError::InvalidKeyIndex),
                }
            }
        }
    }

    /// Returns a result of the clause for a given key.
    ///
    /// # Safety
    /// No check is made on whether a clause is stored by the key.
    /// So, to be used only when there is a guarantee that the clause has not been removed.
    /// E.g., It is always safe to use with binary clauses, but not with long addition clauses, as these may be removed.
    pub unsafe fn get_unchecked(&self, key: &ClauseKey) -> &dbClause {
        match key {
            ClauseKey::OriginalUnit(_) => match self.unit_original.get(key) {
                Some(clause) => clause,
                None => panic!("! Missing clause"),
            },

            ClauseKey::AdditionUnit(_) => {
                //
                match self.unit_addition.get(key) {
                    Some(clause) => clause,
                    None => panic!("! Missing clause"),
                }
            }

            ClauseKey::Original(index) => self.original.get_unchecked(*index as usize),

            ClauseKey::OriginalBinary(index) => self.binary_original.get_unchecked(*index as usize),

            ClauseKey::AdditionBinary(index) => self.binary_addition.get_unchecked(*index as usize),

            ClauseKey::Addition(index, _) => {
                //
                match self.addition.get_unchecked(*index as usize) {
                    Some(clause) => clause,

                    None => panic!("! Missing clause"),
                }
            }
        }
    }

    /// Returns Ok(mutable clause) corresponding to the given key, or an Err(issue) otherwise.
    ///
    /// # Safety
    /// Does not check for a clause, nor the token of a addition key.
    pub unsafe fn get_unchecked_mut(&mut self, key: &ClauseKey) -> &mut dbClause {
        match key {
            ClauseKey::OriginalUnit(_) => {
                //
                match self.unit_original.get_mut(key) {
                    Some(clause) => clause,
                    None => panic!("! Missing clause"),
                }
            }

            ClauseKey::AdditionUnit(_) => {
                //
                match self.unit_addition.get_mut(key) {
                    Some(clause) => clause,
                    None => panic!("! Missing clause"),
                }
            }
            ClauseKey::Original(index) => self.original.get_unchecked_mut(*index as usize),

            ClauseKey::OriginalBinary(index) => {
                self.binary_original.get_unchecked_mut(*index as usize)
            }

            ClauseKey::AdditionBinary(index) => {
                self.binary_addition.get_unchecked_mut(*index as usize)
            }

            ClauseKey::Addition(index, _) => {
                //
                match self.addition.get_unchecked_mut(*index as usize) {
                    Some(clause) => clause,

                    None => panic!("! Missing clause"),
                }
            }
        }
    }
}
