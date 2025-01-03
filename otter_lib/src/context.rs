//! The context --- to which formulas are added and within which solves take place, etc.
//!
//! Strictly, a [GenericContext] and a [Context].
//!
//! The generic context is designed to be generic over various parameters.
//! Though, for the moment this is limited to the source of randomness.
//!
//! Still, this helps distinguish generic context methods against those intended for external use or a particular application.
//! In particular, [from_config](Context::from_config) is implemented for a context rather than a generic context to avoid requiring a source of randomness to be supplied alongside a config.
//!
//! # Example
//! ```rust
//! # use otter_lib::context::Context;
//! # use otter_lib::config::Config;
//! # use otter_lib::dispatch::library::report::{self};
//! let mut the_context = Context::from_config(Config::default(), None);
//!
//! let p_q_clause = the_context.clause_from_string("p q").unwrap();
//! assert!(the_context.add_clause(p_q_clause).is_ok());
//!
//! let not_p = the_context.literal_from_string("-p").expect("oh");
//!
//! assert!(the_context.add_clause(not_p).is_ok());
//! assert!(the_context.solve().is_ok());
//! assert_eq!(the_context.report(), report::Solve::Satisfiable);
//!
//! let the_valuation = the_context.atom_db.valuation_string();
//! assert!(the_valuation.contains("-p"));
//! assert!(the_valuation.contains("q"));
//! ```

use crate::{
    config::Config,
    db::{
        atom::AtomDB, clause::ClauseDB, consequence_q::ConsequenceQ, dbStatus, literal::LiteralDB,
    },
    dispatch::{
        library::report::{self},
        Dispatch,
    },
    generic::minimal_pcg::MinimalPCG32,
};

use rand::SeedableRng;
use std::{rc::Rc, time::Duration};

/// Counts for various things which count, roughly.
pub struct Counters {
    /// A count of every conflict seen during a solve.
    pub total_conflicts: usize,

    /// A count of conflicts seen since the last restart.
    ///
    /// As u32 rather than a usize for easier interaction with scheduling variables.
    pub fresh_conflicts: u32,

    /// A count of every choice/decision made.
    pub total_choices: usize,

    /// The total number of iterations through a solve.
    pub total_iterations: usize,

    /// The number of restarts through a solve.
    pub restarts: usize,

    /// The time taken during a solve.
    pub time: Duration,

    /// The current element in the luby sequence.
    pub luby: crate::generic::luby::Luby,
}

impl Default for Counters {
    fn default() -> Self {
        Counters {
            fresh_conflicts: 0,
            total_choices: 0,
            total_iterations: 0,
            restarts: 0,
            time: Duration::from_secs(0),
            total_conflicts: 0,

            luby: crate::generic::luby::Luby::default(),
        }
    }
}

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

/// A context which uses [MinimalPCG32] as a source of randomness.
pub type Context = GenericContext<MinimalPCG32>;

impl Context {
    /// Creates a context from some given configuration.
    pub fn from_config(config: Config, dispatcher: Option<Rc<dyn Fn(Dispatch)>>) -> Self {
        Self {
            status: dbStatus::Unknown,

            counters: Counters::default(),

            literal_db: LiteralDB::new(dispatcher.clone()),
            clause_db: ClauseDB::new(&config, dispatcher.clone()),
            atom_db: AtomDB::new(&config, dispatcher.clone()),
            consequence_q: ConsequenceQ::default(),

            config,
            dispatcher,

            rng: crate::generic::minimal_pcg::MinimalPCG32::from_seed(0_u64.to_le_bytes()),
        }
    }
}
