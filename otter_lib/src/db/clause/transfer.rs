use crate::{
    db::{keys::ClauseKey, variable::VariableDB},
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
        variables: &mut VariableDB,
    ) -> Result<ClauseKey, err::ClauseDB> {
        match key {
            ClauseKey::Binary(_) => {
                log::error!(target: targets::TRANSFER, "Attempt to transfer binary");
                Err(err::ClauseDB::TransferBinary)
            }
            ClauseKey::Formula(_) | ClauseKey::Learned(_, _) => {
                let the_clause = self.get_mut(key)?;
                the_clause.deactivate();
                let copied_clause = the_clause.to_vec();

                if copied_clause.len() != 2 {
                    log::error!(target: targets::TRANSFER, "Attempt to transfer binary");
                    return Err(err::ClauseDB::TransferBinary);
                }

                let b_key = self.new_binary_id()?;

                unsafe {
                    variables.remove_watch(copied_clause.get_unchecked(0), key)?;
                    variables.remove_watch(copied_clause.get_unchecked(1), key)?;
                }

                if let Some(dispatch) = &self.dispatcher {
                    let delta = delta::ClauseDB::ClauseStart;
                    dispatch(Dispatch::Delta(Delta::ClauseDB(delta)));
                    for literal in &copied_clause {
                        let delta = delta::ClauseDB::ClauseLiteral(*literal);
                        dispatch(Dispatch::Delta(Delta::ClauseDB(delta)));
                    }
                    let delta = delta::ClauseDB::TransferBinary(key, b_key);
                    dispatch(Dispatch::Delta(Delta::ClauseDB(delta)));
                }

                let binary_clause = dbClause::from(b_key, copied_clause, variables);

                self.binary.push(binary_clause);

                if matches!(key, ClauseKey::Learned(_, _)) {
                    self.remove_from_learned(key.index())?;
                }

                Ok(b_key)
            }
        }
    }
}
