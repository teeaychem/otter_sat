use std::collections::HashSet;

use crate::{
    context::{ContextState, GenericContext},
    db::ClauseKey,
    structures::{clause::Clause, literal::Literal},
};

impl<R: rand::Rng + std::default::Default> GenericContext<R> {
    pub fn core_keys(&self) -> Vec<ClauseKey> {
        let ContextState::Unsatisfiable(key) = self.state else {
            todo!("Error path");
        };

        let mut core: HashSet<ClauseKey> = HashSet::default();

        match key {
            ClauseKey::Unit(_) | ClauseKey::Binary(_) | ClauseKey::Original(_) => {
                core.insert(key);
            }
            _ => {}
        }

        let final_clause = self
            .clause_db
            .get_db_clause(&key)
            .expect("Final clause missing");

        let conflict_origins = final_clause.origins();
        if !conflict_origins.is_empty() {
            core.extend(conflict_origins.iter());
        }

        for literal in final_clause.literals() {
            let literal_key = ClauseKey::Unit(literal.negate());
            core.insert(literal_key);

            match self.clause_db.get_db_clause(&literal_key) {
                Err(e) => panic!("Missing core key: {e:?}"),
                Ok(clause) => {
                    let literal_origins = clause.origins();

                    if !literal_origins.is_empty() {
                        core.extend(literal_origins);
                    }
                }
            }
        }

        core.iter().cloned().collect::<Vec<_>>()
    }
}
