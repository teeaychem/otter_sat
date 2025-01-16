//! Macros for sending dispatches from a context.

/// A macro to send bcp dispatches.
/// Requires an optional dispatch method is available via self.
macro_rules! dispatch_bcp_delta {
    ($self:ident, $variant:ident, $literal:expr, $clause:expr ) => {
        if let Some(dispatcher) = &$self.dispatcher {
            let delta = delta::BCP::$variant {
                literal: $literal,
                clause: $clause,
            };
            dispatcher(Dispatch::Delta(Delta::BCP(delta)));
        }
    };
}
pub(crate) use dispatch_bcp_delta;

/// A macro to simplify dispatches.
macro_rules! dispatch_stats {
    ($self:ident ) => {
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

/// A macro to signify a solve has finished.
macro_rules! dispatch_finish {
    ($self:ident) => {
        if let Some(dispatcher) = &$self.dispatcher {
            dispatcher(Dispatch::Report(Report::Finish));
        }
    };
}
pub(crate) use dispatch_finish;

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

/// A macro to help send deltas from the resolution buffer.
///
/// Deltas are often grouped, and so multiple checks on whether a dispatcher is present may be avoided by a different approach.
macro_rules! dispatch_atom_db_delta {
    ( $self:ident, $dispatch:expr ) => {
        if let Some(dispatcher) = &$self.dispatcher {
            dispatcher(Dispatch::Delta(delta::Delta::AtomDB($dispatch)));
        }
    };
}
pub(crate) use dispatch_atom_db_delta;

/// For removing a clause
///
/// Assumes no further use will be made of the clause and calls `into_iter` to access the literals of the clause.
macro_rules! dispatch_clause_removal {
    ($self:ident, $clause:expr) => {
        if let Some(dispatcher) = &$self.dispatcher {
            let delta = delta::ClauseDB::ClauseStart;
            dispatcher(Dispatch::Delta(Delta::ClauseDB(delta)));
            for literal in $clause.into_iter() {
                let delta = delta::ClauseDB::ClauseLiteral(*literal);
                dispatcher(Dispatch::Delta(Delta::ClauseDB(delta)));
            }
            let delta = delta::ClauseDB::Deletion($clause.key());
            dispatcher(Dispatch::Delta(Delta::ClauseDB(delta)));
        }
    };
}
pub(crate) use dispatch_clause_removal;
