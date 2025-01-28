//! Refresh a context, for a new solve, if possible.

use crate::context::{ContextState, GenericContext};

impl<R: rand::Rng + std::default::Default> GenericContext<R> {
    pub fn refresh(&mut self) -> bool {
        match self.state {
            ContextState::Configuration | ContextState::Input => false,

            ContextState::Satisfiable | ContextState::Unsatisfiable(_) | ContextState::Solving => {
                self.backjump(0);
                self.remove_assumptions();
                self.consequence_q.clear();
                self.state = ContextState::Input;
                true
            }
        }
    }
}
