use crate::{
    db::{atom::AtomDB, keys::ClauseKey},
    dispatch::{
        library::delta::{self, Delta},
        Dispatch,
    },
    misc::log::targets::{self},
    types::err::{self},
};

use super::{stored::dbClause, ClauseDB};

impl ClauseDB {
    pub(super) fn transfer_to_binary(
        &mut self,
        key: ClauseKey,
        atoms: &mut AtomDB,
    ) -> Result<ClauseKey, err::ClauseDB> {
        match key {
            ClauseKey::Unit(_) => {
                log::error!(target: targets::TRANSFER, "Attempt to transfer unit");
                Err(err::ClauseDB::TransferUnit)
            }
            ClauseKey::Binary(_) => {
                log::error!(target: targets::TRANSFER, "Attempt to transfer binary");
                Err(err::ClauseDB::TransferBinary)
            }
            ClauseKey::Original(_) | ClauseKey::Addition(_, _) => {
                let the_clause = self.get_mut(key)?;
                the_clause.deactivate();
                let copied_clause = the_clause.to_vec();

                if copied_clause.len() != 2 {
                    log::error!(target: targets::TRANSFER, "Attempt to transfer binary");
                    return Err(err::ClauseDB::TransferBinary);
                }

                let b_key = self.new_binary_id()?;

                unsafe {
                    atoms.remove_watch(copied_clause.get_unchecked(0), key)?;
                    atoms.remove_watch(copied_clause.get_unchecked(1), key)?;
                }

                if let Some(dispatch) = &self.dispatcher {
                    let delta = delta::ClauseDB::ClauseStart;
                    dispatch(Dispatch::Delta(Delta::ClauseDB(delta)));
                    for literal in &copied_clause {
                        let delta = delta::ClauseDB::ClauseLiteral(*literal);
                        dispatch(Dispatch::Delta(Delta::ClauseDB(delta)));
                    }
                    let delta = delta::ClauseDB::Transfer(key, b_key);
                    dispatch(Dispatch::Delta(Delta::ClauseDB(delta)));
                }

                let binary_clause = dbClause::from(b_key, copied_clause, atoms);

                self.binary.push(binary_clause);

                if matches!(key, ClauseKey::Addition(_, _)) {
                    self.remove_from_learned(key.index())?;
                }

                Ok(b_key)
            }
        }
    }
}
