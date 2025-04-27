use crate::{
    config::Config,
    context::Context,
    structures::{clause::CClause, literal::CLiteral},
};

/// A structure which bundles a context with some structures to the IPASIR API.
pub struct ContextBundle {
    /// A context.
    pub context: Context,

    /// Assumptions held.
    pub assumptions: Vec<CLiteral>,

    /// A buffer to hold the literals of a clause being added to the solver.
    pub clause_buffer: CClause,

    /// The literals which occur in the unsatisfiable core identified by [core_keys](ContextBundle::core_keys).
    pub failed_literals: std::collections::HashSet<CLiteral>,
}

impl ContextBundle {
    /// Refreshes the bundled context and clears bundled structures if the context was not already fresh.
    pub fn keep_fresh(&mut self) {
        match self.context.refresh() {
            true => {
                self.failed_literals.clear();
            }
            false => {}
        }
    }
}

impl Default for ContextBundle {
    fn default() -> Self {
        ContextBundle {
            context: Context::from_config(Config::default()),
            assumptions: Vec::default(),
            clause_buffer: Vec::default(),
            failed_literals: std::collections::HashSet::default(),
        }
    }
}
