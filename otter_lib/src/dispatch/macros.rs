//! Macros for sending dispatches.
//!
//!

/// A macro to send bcp dispatches.
/// Requires an optional dispatch method is available via self.
macro_rules! send_bcp {
    ($self:ident, $variant:ident, $literal:expr, $clause:expr ) => {{
        if let Some(dispatcher) = &$self.dispatcher {
            let delta = delta::BCP::$variant {
                literal: $literal,
                clause: $clause,
            };
            dispatcher(Dispatch::Delta(Delta::BCP(delta)));
        }
    }};
}
pub(crate) use send_bcp;

/// A macro to simplify dispatches.
macro_rules! send_stats {
    ($self:ident ) => {{
        if let Some(dispatcher) = &$self.dispatcher {
            let total_iterations = $self.counters.total_iterations;
            let total_choices = $self.counters.total_choices;
            let total_conflicts = $self.counters.total_conflicts;

            dispatcher(Dispatch::Stat(Stat::Iterations(total_iterations)));
            dispatcher(Dispatch::Stat(Stat::Chosen(total_choices)));
            dispatcher(Dispatch::Stat(Stat::Conflicts(total_conflicts)));
            dispatcher(Dispatch::Stat(Stat::Time($self.counters.time)));
        }
    }};
}
pub(crate) use send_stats;

/// A macro to signify a solve has finished.
macro_rules! send_finish {
    ($self:ident) => {
        if let Some(dispatcher) = &$self.dispatcher {
            dispatcher(Dispatch::Report(Report::Finish));
        }
    };
}
pub(crate) use send_finish;
