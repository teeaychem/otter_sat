/*!
Macros for sending dispatches from a context.
*/

/// A macro to simplify dispatches.
macro_rules! dispatch_stats {
    ($self:ident) => {
        if let Some(dispatcher) = &$self.dispatcher {
            let total_iterations = $self.counters.total_iterations;
            let total_decisions = $self.counters.total_decisions;
            let total_conflicts = $self.counters.total_conflicts;

            dispatcher(Dispatch::Stat(Stat::Iterations(total_iterations)));
            dispatcher(Dispatch::Stat(Stat::Chosen(total_decisions)));
            dispatcher(Dispatch::Stat(Stat::Conflicts(total_conflicts)));
            dispatcher(Dispatch::Stat(Stat::Time($self.counters.time)));
        }
    };
}
pub(crate) use dispatch_stats;
