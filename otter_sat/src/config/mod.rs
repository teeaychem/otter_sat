/*!
Configuration of a context.

Primary configuration is context.
All configuration for a context are contained within context.
Some structures clone parts of the configuration.
Databases.

*/
use dbs::{AtomDBConfig, ClauseDBConfig};
use vsids::VSIDS;

mod config_option;
pub use config_option::ConfigOption;

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

use crate::{
    context::ContextState,
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
    pub luby_u: ConfigOption<generic::luby::LubyRepresentation>,

    /// The probability of assigning positive polarity to a atom when freely choosing a atom.
    pub polarity_lean: ConfigOption<PolarityLean>,

    /// Preprocessing configuration
    pub random_decision_bias: ConfigOption<RandomDecisionBias>,

    /// A scheduler for things such as restarts and reductions.
    pub scheduler: Scheduler,

    /// Which stopping criteria to use during resolution based analysis
    pub stopping_criteria: ConfigOption<StoppingCriteria>,

    /// Default to th last set value of a atom when choosing  a value for the atom, otherwise decision with specified probability.
    pub phase_saving: ConfigOption<bool>,

    /// Enable preprocessing of 𝐅.
    pub preprocessing: ConfigOption<bool>,

    /// Permit (scheduled) restarts.
    pub restart: ConfigOption<bool>,

    /// Permit subsumption of formulas.
    pub subsumption: ConfigOption<bool>,

    /// The time limit for a solve.
    pub time_limit: ConfigOption<std::time::Duration>,

    /// Which VSIDS variant to use during resolution based analysis
    pub vsids_variant: ConfigOption<VSIDS>,

    pub stacked_assumptions: ConfigOption<bool>,
}

impl Default for Config {
    /// The default context is (roughly) configured to provide quick, deterministic, results on a library of tests.
    fn default() -> Self {
        Config {
            atom_db: AtomDBConfig::default(),
            clause_db: ClauseDBConfig::default(),
            literal_db: LiteralDBConfig::default(),

            luby_u: ConfigOption {
                name: "luby",
                min: generic::luby::LubyRepresentation::MIN,
                max: generic::luby::LubyRepresentation::MAX,
                max_state: ContextState::Configuration,
                value: 128,
            },

            polarity_lean: ConfigOption {
                name: "polarity_lean",
                min: PolarityLean::MIN,
                max: PolarityLean::MAX,
                max_state: ContextState::Configuration,
                value: 0.0,
            },

            random_decision_bias: ConfigOption {
                name: "random_decision_bias",
                min: PolarityLean::MIN,
                max: PolarityLean::MAX,
                max_state: ContextState::Configuration,
                value: 0.0,
            },

            scheduler: Scheduler {
                luby: Some(2),
                conflict: Some(50_000),
            },

            stopping_criteria: ConfigOption {
                name: "stopping_criteria",
                min: StoppingCriteria::MIN,
                max: StoppingCriteria::MAX,
                max_state: ContextState::Configuration,
                value: StoppingCriteria::FirstUIP,
            },

            phase_saving: ConfigOption {
                name: "phase_saving",
                min: false,
                max: true,
                max_state: ContextState::Configuration,
                value: true,
            },

            preprocessing: ConfigOption {
                name: "preprocessing",
                min: false,
                max: true,
                max_state: ContextState::Configuration,
                value: false,
            },

            restart: ConfigOption {
                name: "restart",
                min: false,
                max: true,
                max_state: ContextState::Configuration,
                value: true,
            },

            subsumption: ConfigOption {
                name: "subsumption",
                min: false,
                max: true,
                max_state: ContextState::Configuration,
                value: false,
            },

            time_limit: ConfigOption {
                name: "time_limit",
                min: std::time::Duration::from_secs(0),
                max: std::time::Duration::MAX,
                max_state: ContextState::Configuration,
                value: std::time::Duration::from_secs(0),
            },

            vsids_variant: ConfigOption {
                name: "vsids",
                min: VSIDS::MIN,
                max: VSIDS::MAX,
                max_state: ContextState::Configuration,
                value: VSIDS::MiniSAT,
            },

            stacked_assumptions: ConfigOption {
                name: "stacked_assumptions",
                min: false,
                max: true,
                max_state: ContextState::Configuration,
                value: false,
            },
        }
    }
}
