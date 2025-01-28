use crate::{
    config::Config,
    db::{
        atom::AtomDB, clause::ClauseDB, consequence_q::ConsequenceQ, literal::LiteralDB, ClauseKey,
    },
    dispatch::{
        library::report::{self},
        Dispatch,
    },
    ipasir::IpasirCallbacks,
};

use std::rc::Rc;

use super::{ContextState, Counters};

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
/// let dispatcher = None;
/// let context = GenericContext::<MinimalPCG32>::from_config(Config::default(), dispatcher);
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

    /// An optional function to send dispatches with.
    pub dispatcher: Option<Rc<dyn Fn(Dispatch)>>,

    pub ipasir_callbacks: Option<IpasirCallbacks>,
}

impl<R: rand::Rng + std::default::Default> GenericContext<R> {
    pub fn report(&self) -> report::SolveReport {
        use crate::context::ContextState;
        match self.state {
            ContextState::Configuration | ContextState::Input | ContextState::Solving => {
                report::SolveReport::Unknown
            }
            ContextState::Satisfiable => report::SolveReport::Satisfiable,
            ContextState::Unsatisfiable(_) => report::SolveReport::Unsatisfiable,
        }
    }

    pub fn unsatisfiable_clause(&self) -> Result<ClauseKey, ()> {
        match self.state {
            ContextState::Unsatisfiable(key) => Ok(key),
            _ => Err(()),
        }
    }
}
