use crate::{
    context::{ContextState, GenericContext},
    db::ClauseKey,
};

impl<R: rand::Rng + std::default::Default> GenericContext<R> {
    pub fn note_conflict(&mut self, key: ClauseKey) {
        self.state = ContextState::Unsatisfiable(key);
        let clause = unsafe { self.clause_db.get_unchecked_mut(&key).clone() };
        self.clause_db.make_callback_unsatisfiable(&clause);
    }
}
