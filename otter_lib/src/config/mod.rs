//! Configuration of a context.
//!
//! Primary configuration is context.
//! All configuration for a context are contained within context.
//! Some structures clone parts of the configuration.
//! Databases.
//!
use dbs::{ClauseDBConfig, VariableDBConfig};
use misc::switches::Switches;

pub mod dbs;
pub mod misc;

#[derive(Clone, Debug)]
pub struct Config {
    /// The `u` value to multiply the luby sequence by when determining whether to perform a restart.
    pub luby_u: LubyRepresentation,

    /// The probability of assigning positive polarity to a variable when freely choosing a variable.
    pub polarity_lean: PolarityLean,

    /// Preprocessing configuration
    pub random_choice_frequency: RandomChoiceFrequency,

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

    /// Configuration of the variable database.
    pub variable_db: VariableDBConfig,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            luby_u: 128,

            polarity_lean: 0.0,

            random_choice_frequency: 0.0,

            stopping_criteria: StoppingCriteria::FirstUIP,

            scheduler: Scheduler {
                luby: Some(2),
                conflict: Some(50_000),
            },

            time_limit: None,
            vsids_variant: VSIDS::MiniSAT,

            switch: Switches::default(),
            clause_db: ClauseDBConfig::default(),
            variable_db: VariableDBConfig::default(),
        }
    }
}

/// Representation used for clause and variable activity
pub type Activity = f64;

/// Glue / literal block distance
pub type GlueStrength = u8;

/// Representation used for generating the luby sequence
pub type LubyRepresentation = u32;

/// Representation for the probability of choosing `true`
pub type PolarityLean = f64;

/// Representation for the probability of making a random choice
pub type RandomChoiceFrequency = f64;

/// Scheduler for reductions.
/// If two scheduled reductions coincide, only one reduction takes place.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Scheduler {
    /// Reuce the clause database every `luby` times a luby interrupt happens.
    pub luby: Option<u32>,

    /// Reuce the clause database every `conflict` conflicts.
    pub conflict: Option<u32>,
}

/// Variant stopping criterias to use during resolution-based analysis.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum StoppingCriteria {
    /// Stop at the first unique implication point.
    ///
    /// In other words, apply resolution until the clause obtained by resolution is asserting on the current valuation without the last choice made, and any consequences of that choice.
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

/// Variant way to apply VSIDS (variable state independent decay sum) during during resolution-based analysis.
#[derive(Clone, Copy, Debug)]
#[allow(clippy::upper_case_acronyms)]
pub enum VSIDS {
    /// When learning a clause by applying resolution to a sequence of clauses every variable occurring in the learnt clause is bumped.
    Chaff,
    /// When learning a clause by applying resolution to a sequence of clauses every variable occurring in some clause used during resolution (including the learnt clause) is bumped.
    MiniSAT,
}

impl std::fmt::Display for VSIDS {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Chaff => write!(f, "Chaff"),
            Self::MiniSAT => write!(f, "MiniSAT"),
        }
    }
}
