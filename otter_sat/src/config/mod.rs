//! Configuration of a context.
//!
//! Primary configuration is context.
//! All configuration for a context are contained within context.
//! Some structures clone parts of the configuration.
//! Databases.
//!
use dbs::{AtomDBConfig, ClauseDBConfig};
use vsids::VSIDS;

pub mod dbs;

pub mod vsids;

mod activity;
pub use activity::Activity;

mod lbd;
pub use lbd::LBD;

mod rng;
pub use rng::{PolarityLean, RandomDecisionBias};

mod scheduler;
pub use scheduler::Scheduler;

mod stopping_criteria;
pub use stopping_criteria::StoppingCriteria;

mod switches;
pub use switches::Switches;

use crate::{
    db::literal::config::LiteralDBConfig,
    generic::{self},
};

/// The primary configuration structure.
#[derive(Clone)]
pub struct Config {
    /// Configuration of the atom database.
    pub atom_db: AtomDBConfig,

    /// Configuration of the clause database.
    pub clause_db: ClauseDBConfig,

    pub literal_db: LiteralDBConfig,

    /// The `u` value to multiply the luby sequence by when determining whether to perform a restart.
    pub luby_u: generic::luby::LubyRepresentation,

    /// The probability of assigning positive polarity to a atom when freely choosing a atom.
    pub polarity_lean: PolarityLean,

    /// Preprocessing configuration
    pub random_decision_bias: RandomDecisionBias,

    /// A scheduler for things such as restarts and reductions.
    pub scheduler: Scheduler,

    /// Which stopping criteria to use during resolution based analysis
    pub stopping_criteria: StoppingCriteria,

    /// Configurations switched on (or off).
    pub switch: Switches,

    /// The time limit for a solve.
    pub time_limit: Option<std::time::Duration>,

    /// Which VSIDS variant to use during resolution based analysis
    pub vsids_variant: VSIDS,

    pub stacked_assumptions: bool,
}

impl Default for Config {
    /// The default context is (roughly) configured to provide quick, deterministic, results on a library of tests.
    fn default() -> Self {
        Config {
            atom_db: AtomDBConfig::default(),
            clause_db: ClauseDBConfig::default(),
            literal_db: LiteralDBConfig::default(),

            luby_u: 128,

            polarity_lean: 0.0,

            random_decision_bias: 0.0,

            scheduler: Scheduler {
                luby: Some(2),
                conflict: Some(50_000),
            },

            stopping_criteria: StoppingCriteria::FirstUIP,

            switch: Switches::default(),

            time_limit: None,
            vsids_variant: VSIDS::MiniSAT,

            stacked_assumptions: true,
        }
    }
}
