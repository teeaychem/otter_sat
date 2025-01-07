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
#[doc(hidden)]
pub mod misc;
pub mod vsids;

/// The primary configuration structure.
#[derive(Clone)]
pub struct Config {
    /// The `u` value to multiply the luby sequence by when determining whether to perform a restart.
    pub luby_u: crate::generic::luby::LubyRepresentation,

    /// The probability of assigning positive polarity to a atom when freely choosing a atom.
    pub polarity_lean: PolarityLean,

    /// Preprocessing configuration
    pub random_decision_bias: RandomDecisionBias,

    /// Which stopping criteria to use during resolution based analysis
    pub stopping_criteria: StoppingCriteria,

    /// The time limit for a solve.
    pub time_limit: Option<std::time::Duration>,

    /// Which VSIDS variant to use during resolution based analysis
    pub vsids_variant: VSIDS,

    /// A scheduler for things such as restarts and reductions.
    pub scheduler: Scheduler,

    /// Configurations switched on (or off).
    pub switch: Switches,

    /// Configuration of the clause database.
    pub clause_db: ClauseDBConfig,

    /// Configuration of the atom database.
    pub atom_db: AtomDBConfig,
}

impl Default for Config {
    /// The default context is (roughly) configured to provide quick, deterministic, results on a library of tests.
    fn default() -> Self {
        Config {
            luby_u: 128,

            polarity_lean: 0.0,

            random_decision_bias: 0.0,

            stopping_criteria: StoppingCriteria::FirstUIP,

            scheduler: Scheduler {
                restart: Some(2),
                conflict: Some(50_000),
            },

            time_limit: None,
            vsids_variant: VSIDS::MiniSAT,

            switch: Switches::default(),
            clause_db: ClauseDBConfig::default(),
            atom_db: AtomDBConfig::default(),
        }
    }
}

/// Representation used for clause and atom activity.
pub type Activity = f64;

/// Literal block distance, a.k.a 'glue'.
///
/// See [On the Glucose SAT Solver](https://dx.doi.org/10.1142/S0218213018400018) for an overview of LBD, and roughly a decade's worth of insight into the metric.
pub type LBD = u8;

/// Representation for the probability of choosing `true`
pub type PolarityLean = f64;

/// Representation for the probability of making a random decision
pub type RandomDecisionBias = f64;

/// Schedulers, for reduction of the clause database, etc.
///
/// Note: If two scheduled reductions coincide, only one reduction takes place.
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Scheduler {
    /// Reuce the clause database every `luby` times a luby interrupt happens.
    pub restart: Option<u32>,

    /// Reuce the clause database every `conflict` conflicts.
    pub conflict: Option<u32>,
}

/// Variant stopping criterias to use during resolution-based analysis.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum StoppingCriteria {
    /// Stop at the first unique implication point.
    ///
    /// In other words, apply resolution until the clause obtained by resolution is asserting on the current valuation without the last decision made, and any consequences of that decision.
    FirstUIP,
    /// Apply resolution to each clause in the sequence of clauses.
    None,
}

impl std::fmt::Display for StoppingCriteria {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::FirstUIP => write!(f, "FirstUIP"),
            Self::None => write!(f, "None"),
        }
    }
}

/// Boolean valued context configurations
///
/// When set to true things related to the identifier are enabled.
#[derive(Clone)]
pub struct Switches {
    /// Default to th last set value of a atom when choosing  a value for the atom, otherwise decision with specified probability.
    pub phase_saving: bool,

    /// Enable preprocessing of 𝐅.
    pub preprocessing: bool,

    /// Permit (scheduled) restarts.
    pub restart: bool,

    /// Permit subsumption of formulas.
    pub subsumption: bool,
}

impl Default for Switches {
    fn default() -> Self {
        Switches {
            phase_saving: true,
            preprocessing: false,
            restart: true,
            subsumption: true,
        }
    }
}
