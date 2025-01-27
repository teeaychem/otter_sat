use std::collections::HashSet;

use crate::{
    db::{
        atom::AtomDB,
        clause::{db_clause::dbClause, ClauseDB},
        keys::ClauseKey,
    },
    dispatch::{
        library::delta::{self, Delta},
        Dispatch,
    },
    misc::log::targets::{self},
    structures::literal::Literal,
    types::err::{self},
};

impl ClauseDB {
    /// Transfers an binary original or addition clause which is not stored in the binary database to the binary database, if possible.
    ///
    /// To be used after some operation which may shorten a clause, such as [subsumption](dbClause::subsume).
    ///
    /// On success a key to the binary clause is returned.
    ///
    /// ```rust, ignore
    /// clause.subsume(literal, ...)?;
    /// let Ok(new_key) = self.transfer_to_binary(old_key, atom_db)
    /// ```
    /*
    Addition clauses are removed from the database, but as there is at present no way to remove original clauses, these are ignored.
     */
    // TODO: Cleanup original/addition repetition
    pub fn transfer_to_binary(
        &mut self,
        key: ClauseKey,
        atom_db: &mut AtomDB,
        premises: HashSet<ClauseKey>,
    ) -> Result<ClauseKey, err::ClauseDBError> {
        match key {
            ClauseKey::OriginalUnit(_) | ClauseKey::AdditionUnit(_) => {
                log::error!(target: targets::TRANSFER, "Attempt to transfer unit");
                Err(err::ClauseDBError::TransferUnit)
            }

            ClauseKey::OriginalBinary(_) | ClauseKey::AdditionBinary(_) => {
                log::error!(target: targets::TRANSFER, "Attempt to transfer binary");
                Err(err::ClauseDBError::TransferBinary)
            }

            ClauseKey::Original(_) => {
                let the_clause = self.get_mut(&key)?;
                the_clause.deactivate();
                let copied_clause = the_clause.to_vec();

                if copied_clause.len() != 2 {
                    log::error!(target: targets::TRANSFER, "Attempt to transfer binary");
                    return Err(err::ClauseDBError::TransferBinary);
                }

                let binary_key = self.fresh_original_binary_key()?;

                unsafe {
                    // Ok, as checked length is 2, above.
                    let zero = copied_clause.get_unchecked(0);
                    atom_db.unwatch_unchecked(zero.atom(), zero.polarity(), &key)?;
                    let one = copied_clause.get_unchecked(1);
                    atom_db.unwatch_unchecked(one.atom(), one.polarity(), &key)?;
                }

                if let Some(dispatch) = &self.dispatcher {
                    let delta = delta::ClauseDB::ClauseStart;
                    dispatch(Dispatch::Delta(Delta::ClauseDB(delta)));
                    for literal in &copied_clause {
                        let delta = delta::ClauseDB::ClauseLiteral(*literal);
                        dispatch(Dispatch::Delta(Delta::ClauseDB(delta)));
                    }
                    let delta = delta::ClauseDB::Transfer(key, binary_key);
                    dispatch(Dispatch::Delta(Delta::ClauseDB(delta)));
                }

                let binary_clause =
                    dbClause::new_nonunit(binary_key, copied_clause, atom_db, None, premises);

                self.binary_original.push(binary_clause);

                if matches!(key, ClauseKey::Addition(_, _)) {
                    self.remove_addition(key.index())?;
                }

                Ok(binary_key)
            }

            ClauseKey::Addition(_, _) => {
                let the_clause = self.get_mut(&key)?;
                the_clause.deactivate();
                let copied_clause = the_clause.to_vec();

                if copied_clause.len() != 2 {
                    log::error!(target: targets::TRANSFER, "Attempt to transfer binary");
                    return Err(err::ClauseDBError::TransferBinary);
                }

                let binary_key = self.fresh_addition_binary_key()?;

                unsafe {
                    // Ok, as checked length is 2, above.
                    let zero = copied_clause.get_unchecked(0);
                    atom_db.unwatch_unchecked(zero.atom(), zero.polarity(), &key)?;
                    let one = copied_clause.get_unchecked(1);
                    atom_db.unwatch_unchecked(one.atom(), one.polarity(), &key)?;
                }

                if let Some(dispatch) = &self.dispatcher {
                    let delta = delta::ClauseDB::ClauseStart;
                    dispatch(Dispatch::Delta(Delta::ClauseDB(delta)));
                    for literal in &copied_clause {
                        let delta = delta::ClauseDB::ClauseLiteral(*literal);
                        dispatch(Dispatch::Delta(Delta::ClauseDB(delta)));
                    }
                    let delta = delta::ClauseDB::Transfer(key, binary_key);
                    dispatch(Dispatch::Delta(Delta::ClauseDB(delta)));
                }

                let binary_clause =
                    dbClause::new_nonunit(binary_key, copied_clause, atom_db, None, premises);

                self.binary_addition.push(binary_clause);

                if matches!(key, ClauseKey::Addition(_, _)) {
                    self.remove_addition(key.index())?;
                }

                Ok(binary_key)
            }
        }
    }
}
