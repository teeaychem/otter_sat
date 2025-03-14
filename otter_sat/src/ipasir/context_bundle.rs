use std::collections::HashSet;

use crate::{
    config::Config,
    context::Context,
    db::ClauseKey,
    structures::{clause::CClause, literal::CLiteral},
};

/// A structure which bundles a context with some structures to the IPASIR API.
pub struct ContextBundle {
    /// A context.
    pub context: Context,

    pub assumptions: Vec<CLiteral>,

    /// A buffer to hold the literals of a clause being added to the solver.
    pub clause_buffer: CClause,

    /// The keys to a an unsatisfiable core of the formula.
    pub core_keys: Vec<ClauseKey>,

    /// The literals which occur in the unsatisfiable core identified by [core_keys](ContextBundle::core_keys).
    pub failed_literals: HashSet<CLiteral>,
}

impl ContextBundle {
    /// Refreshes the bundled context and clears bundled structures if the context was not already fresh.
    pub fn keep_fresh(&mut self) {
        match self.context.refresh() {
            true => {
                self.core_keys.clear();
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
            core_keys: Vec::default(),
            failed_literals: HashSet::default(),
        }
    }
}
