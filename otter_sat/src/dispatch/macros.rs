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

/// A macro to help send deltas from the resolution buffer.
///
/// Deltas are often grouped, and so multiple checks on whether a dispatcher is present may be avoided by a different approach.
macro_rules! dispatch_resolution_delta {
    ( $self:ident, $dispatch:expr ) => {
        if let Some(dispatcher) = &$self.dispatcher {
            dispatcher(Dispatch::Delta(delta::Delta::Resolution($dispatch)));
        }
    };
}
pub(crate) use dispatch_resolution_delta;

/// Clause db deltas
macro_rules! dispatch_clause_db_delta {
    ($self:ident, $variant:ident, $key:expr) => {
        if let Some(dispatcher) = &$self.dispatcher {
            let delta = delta::ClauseDB::$variant($key);
            dispatcher(Dispatch::Delta(Delta::ClauseDB(delta)));
        }
    };
}
pub(crate) use dispatch_clause_db_delta;
