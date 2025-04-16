/*!
Configuration of a context.

Primary configuration is context.
All configuration for a context are contained within context.
Some structures clone parts of the configuration.
Databases.
*/

use dbs::ClauseDBConfig;
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

mod stopping_criteria;
pub use stopping_criteria::StoppingCriteria;

mod minimization_criteria;
pub use minimization_criteria::MinimizationCriteria;

use crate::{
    context::ContextState,
    generic::{self},
};

/// The primary configuration structure.
#[derive(Clone)]
pub struct Config {
    /// Configuration of the clause database.
    pub clause_db: ClauseDBConfig,

    /// The `u` value to multiply the luby sequence by when determining whether to perform a restart.
    pub luby_u: ConfigOption<generic::luby::LubyRepresentation>,

    /// The probability of assigning positive polarity to a atom when freely choosing a atom.
    pub polarity_lean: ConfigOption<PolarityLean>,

    /// Preprocessing configuration
    pub random_decision_bias: ConfigOption<RandomDecisionBias>,

    /// Which stopping criteria to use during resolution based analysis
    pub stopping_criteria: ConfigOption<StoppingCriteria>,

    /// Default to the last set value of a atom when choosing  a value for the atom, otherwise decision with specified probability.
    pub phase_saving: ConfigOption<bool>,

    /// Enable preprocessing of the formula.
    pub preprocessing: ConfigOption<bool>,

    /// Permit (scheduled) restarts.
    pub restarts: ConfigOption<bool>,

    /// Configuration for minimizing learnt clauses.
    pub minimization: ConfigOption<MinimizationCriteria>,

    /// Permit subsumption of formulas.
    pub subsumption: ConfigOption<bool>,

    /// The time limit for a solve.
    pub time_limit: ConfigOption<std::time::Duration>,

    /// Which VSIDS variant to use during resolution based analysis
    pub vsids: ConfigOption<VSIDS>,

    /// Reuce the clause database every `luby` times a luby interrupt happens.
    pub luby_mod: ConfigOption<u32>,

    /// Reuce the clause database every `conflict` conflicts.
    pub conflict_mod: ConfigOption<u32>,

    /// The amount with which to bump a atom by when applying [VSIDS](crate::config::vsids).
    pub atom_bump: ConfigOption<Activity>,

    /// After a conflict increase the atom bump by a value (proportional to) 1 / (1 - `FACTOR`^-3)
    pub atom_decay: ConfigOption<Activity>,

    /// Whether to stack assumptions on individual levels, or combine all assumptions on a single level.
    pub stacked_assumptions: ConfigOption<bool>,
}

impl Default for Config {
    /// The default context is (roughly) configured to provide quick, deterministic, results on a library of tests.
    fn default() -> Self {
        Config {
            clause_db: ClauseDBConfig::default(),

            luby_u: ConfigOption {
                name: "luby",
                min: 1,
                max: generic::luby::LubyRepresentation::MAX,
                max_state: ContextState::Configuration,
                value: 128,
            },

            polarity_lean: ConfigOption {
                name: "polarity_lean",
                min: 0.0,
                max: 1.0,
                max_state: ContextState::Configuration,
                value: 0.0,
            },

            random_decision_bias: ConfigOption {
                name: "random_decision_bias",
                min: 0.0,
                max: 1.0,
                max_state: ContextState::Configuration,
                value: 0.0,
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

            restarts: ConfigOption {
                name: "restart",
                min: false,
                max: true,
                max_state: ContextState::Configuration,
                value: true,
            },

            minimization: ConfigOption {
                name: "subsumption",
                min: MinimizationCriteria::MIN,
                max: MinimizationCriteria::MAX,
                max_state: ContextState::Configuration,
                value: MinimizationCriteria::Recursive,
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

            vsids: ConfigOption {
                name: "vsids",
                min: VSIDS::MIN,
                max: VSIDS::MAX,
                max_state: ContextState::Configuration,
                value: VSIDS::MiniSAT,
            },

            luby_mod: ConfigOption {
                name: "luby_mod",
                min: u32::MIN,
                max: u32::MAX,
                max_state: ContextState::Configuration,
                value: 2,
            },

            conflict_mod: ConfigOption {
                name: "conflict_mod",
                min: u32::MIN,
                max: u32::MAX,
                max_state: ContextState::Configuration,
                value: 50_000,
            },

            atom_bump: ConfigOption {
                name: "atom_bump",
                min: Activity::MIN,
                max: (2.0 as Activity).powi(512),
                max_state: ContextState::Configuration,
                value: 1.0,
            },

            atom_decay: ConfigOption {
                name: "atom_decay",
                min: Activity::MIN,
                max: Activity::MAX,
                max_state: ContextState::Configuration,
                value: 50.0 * 1e-3,
            },

            stacked_assumptions: ConfigOption {
                name: "stacked_assumptions",
                min: false,
                max: true,
                max_state: ContextState::Configuration,
                value: true,
            },
        }
    }
}
