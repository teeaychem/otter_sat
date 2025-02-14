use crate::{
    config::Config,
    db::{
        atom::AtomDB, clause::ClauseDB, consequence_q::ConsequenceQ, literal::LiteralDB, ClauseKey,
    },
    reports::Report,
    resolution_buffer::ResolutionBuffer,
    types::err::ErrorKind,
};

use super::{callbacks::CallbackTerminate, ContextState, Counters};

/// A generic context, parameratised to a source of randomness.
///
/// Requires a source of [rng](rand::Rng) which (also) implements [Default](std::default::Default).
///
/// [Default](std::default::Default) is used in calls [make_decision](GenericContext::make_decision) to appease the borrow checker, and may be relaxed with a different implementation.
///
/// # Example
///
/// ```rust
/// # use otter_sat::context::GenericContext;
/// # use otter_sat::generic::minimal_pcg::MinimalPCG32;
/// # use otter_sat::config::Config;
/// let context = GenericContext::<MinimalPCG32>::from_config(Config::default());
/// ```
pub struct GenericContext<R: rand::Rng + std::default::Default> {
    /// The configuration of a context.
    pub config: Config,

    /// Counters related to a context/solve.
    pub counters: Counters,

    /// The atom database.
    /// See [db::atom](crate::db::atom) for details.
    pub atom_db: AtomDB,

    /// The clause database.
    /// See [db::clause](crate::db::clause) for details.
    pub clause_db: ClauseDB,

    /// The literal database.
    /// See [db::literal](crate::db::literal) for details.
    pub literal_db: LiteralDB,

    /// The consequence queue.
    /// See [db::consequence_q](crate::db::consequence_q) for details.
    pub consequence_q: ConsequenceQ,

    /// The status of the context.
    pub state: ContextState,

    /// The source of rng.
    pub rng: R,

    /// A buffer for resolution
    pub resolution_buffer: ResolutionBuffer,

    /// Terminates procedures, if true.
    pub(super) callback_terminate: Option<Box<CallbackTerminate>>,
}

impl<R: rand::Rng + std::default::Default> GenericContext<R> {
    /// A report on the state of the context.
    pub fn report(&self) -> Report {
        use crate::context::ContextState;
        match self.state {
            ContextState::Configuration | ContextState::Input | ContextState::Solving => {
                Report::Unknown
            }
            ContextState::Satisfiable => Report::Satisfiable,
            ContextState::Unsatisfiable(_) => Report::Unsatisfiable,
        }
    }

    /// The clause with which unsatisfiability of the context was determined by.
    pub fn unsatisfiable_clause(&self) -> Result<ClauseKey, ErrorKind> {
        match self.state {
            ContextState::Unsatisfiable(key) => Ok(key),
            _ => Err(ErrorKind::InvalidState),
        }
    }
}
