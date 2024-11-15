use crate::{
    db::{keys::ClauseKey, variable::VariableDB},
    dispatch::{
        delta::{self},
        Dispatch,
    },
    misc::log::targets::{self},
    types::err::{self},
};

use super::{stored::StoredClause, ClauseDB};

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

                let delta = delta::ClauseDB::TransferBinary(key, b_key, copied_clause.clone());
                self.tx.send(Dispatch::ClauseDB(delta));

                unsafe {
                    variables.remove_watch(copied_clause.get_unchecked(0), key)?;
                    variables.remove_watch(copied_clause.get_unchecked(1), key)?;
                }

                let binary_clause = StoredClause::from(b_key, copied_clause, variables);

                self.binary.push(binary_clause);

                if matches!(key, ClauseKey::Learned(_, _)) {
                    self.remove_from_learned(key.index())?;
                }

                Ok(b_key)
            }
        }
    }
}
