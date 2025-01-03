use crate::{
    config::Config,
    db::{
        atom::AtomDB, clause::ClauseDB, consequence_q::ConsequenceQ, dbStatus, literal::LiteralDB,
    },
    dispatch::{
        library::report::{self},
        Dispatch,
    },
};

use std::rc::Rc;

use super::Counters;

/// A generic context, parameratised to a source of randomness.
///
/// In addition, the source of rng must also implement default to mitigate limitations of the borrow checker.
pub struct GenericContext<R: rand::Rng + std::default::Default> {
    /// The configuration of a context.
    pub config: Config,

    /// Counters related to a context/solve.
    pub counters: Counters,

    /// The clause database.
    pub clause_db: ClauseDB,

    /// The literal database.
    pub literal_db: LiteralDB,

    /// The atom database.
    pub atom_db: AtomDB,

    /// The consequence queue.
    pub consequence_q: ConsequenceQ,

    /// The status of the context.
    pub status: dbStatus,

    /// The source of rng.
    pub rng: R,

    /// An optional function to send dispatches with.
    pub dispatcher: Option<Rc<dyn Fn(Dispatch)>>,
}

impl<R: rand::Rng + std::default::Default> GenericContext<R> {
    pub fn report(&self) -> report::Solve {
        match self.status {
            dbStatus::Consistent => report::Solve::Satisfiable,
            dbStatus::Inconsistent => report::Solve::Unsatisfiable,
            _ => report::Solve::Unknown,
        }
    }
}

impl std::fmt::Display for dbStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            dbStatus::Consistent => write!(f, "Consistent"),
            Self::Inconsistent => write!(f, "Inconsistent"),
            Self::Unknown => write!(f, "Unknown"),
        }
    }
}
