use crate::context::{ContextState, GenericContext};

impl<R: rand::Rng + std::default::Default> GenericContext<R> {
    /// Refreshes a context by clearning assumptions, the consequence queue, and any decisions, if a solve has started.
    ///
    /// Note, this will not clear any non-stacked assumptions added *unless* [solve](GenericContext::solve) has been called.
    pub fn refresh(&mut self) -> bool {
        match self.state {
            ContextState::Configuration | ContextState::Input => false,

            ContextState::Satisfiable | ContextState::Unsatisfiable(_) | ContextState::Solving => {
                self.backjump(0);
                self.clear_assumptions();

                self.consequence_q.clear();
                self.state = ContextState::Input;
                true
            }
        }
    }
}
