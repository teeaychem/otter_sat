use crate::{
    context::{ContextState, GenericContext},
    db::ClauseKey,
};

impl<R: rand::Rng + std::default::Default> GenericContext<R> {
    pub fn note_conflict(&mut self, key: ClauseKey) {
        match self.state {
            ContextState::Unsatisfiable(_) => {}

            _ => match key {
                ClauseKey::OriginalUnit(0) => {
                    self.state = ContextState::Unsatisfiable(key);
                }

                _ => {
                    self.state = ContextState::Unsatisfiable(key);
                    let clause = match self.clause_db.get_mut(&key) {
                        Err(e) => {
                            println!("The key {key}");
                            panic!("{e:?}");
                        }
                        Ok(c) => c.clone(),
                    };
                    self.clause_db.make_callback_unsatisfiable(&clause);
                }
            },
        }
    }
}
