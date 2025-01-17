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
//! # use otter_sat::context::Context;
//! # use otter_sat::config::Config;
//! # use otter_sat::dispatch::library::report::{self};
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

mod counters;
pub use counters::Counters;
mod generic;
pub use generic::GenericContext;
mod specific;
pub use specific::Context;

#[allow(non_camel_case_types)]
#[derive(PartialEq, Eq)]
/// The state of a context.
/// These states correspond to the states defined in the IPASIR2 specification.
pub enum ContextState {
    Configuration,
    Input,
    /// The database is known to be consistent, e.g. with a complete valuation.
    Satisfiable,
    /// The database is known to be inconsistnet, e.g. with an unsatisfiable clause identified.
    Unsatisfiable,
    /// The consistency of the database is unknown.
    Solving,
}

impl std::fmt::Display for ContextState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Configuration => write!(f, "Configuration"),
            Self::Input => write!(f, "Input"),
            Self::Satisfiable => write!(f, "Satisfiable"),
            Self::Unsatisfiable => write!(f, "Unsatisfiable"),
            Self::Solving => write!(f, "Solving"),
        }
    }
}
