/*!
Procedures for when a conflict is identified.
*/

use crate::{
    context::{ContextState, GenericContext},
    db::ClauseKey,
};

impl<R: rand::Rng + std::default::Default> GenericContext<R> {
    /// Steps to take when the clause indexed by `key` is in conflict with / unsatisfiable on the background assignment.
    ///
    /// In particular, updating the state of the context and making the relevant callbacks (if defined).
    pub fn note_conflict(&mut self, key: ClauseKey) {
        // Note, if the context is already in an unsatisfiable state, the previously stored key is overwritten.
        // Though, generally speaking, there are no paths to stacked conflicts.
        match key {
            ClauseKey::OriginalUnit(0) => {
                self.state = ContextState::Unsatisfiable(key);
            }

            _ => {
                self.state = ContextState::Unsatisfiable(key);
                let clause = match self.clause_db.get_mut(&key) {
                    Err(e) => {
                        panic!("{e:?} with key {key}");
                    }

                    Ok(c) => c.clone(),
                };
                self.clause_db.make_callback_unsatisfiable(&clause);
            }
        }
    }
}
