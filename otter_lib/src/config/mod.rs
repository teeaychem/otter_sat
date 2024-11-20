//! Configuration details.
//!
//! Primary configuration is context.
//! All configuration for a context are contained within context.
//! Some structures clone parts of the configuration.
//! Databases.
//!

pub mod context;
pub mod dbs;
pub mod misc;

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
pub struct ReductionScheduler {
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
