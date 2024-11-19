//! Configuration for a context.

pub mod context;
pub mod dbs;
pub mod misc;

/// Representation used for clause and variable activity
pub type Activity = f64;

/// Glue / literal block distance
pub type GlueStrength = u8;

/// Representation used for generating the luby sequence
pub type LubyRepresentation = u32;

/// Precision
pub type PolarityLean = f64;
pub type RandomChoiceFrequency = f64;

/// Variant stopping criterias to use during resolution-based analysis.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum StoppingCriteria {
    /// Stop at the first unique implication point.
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
#[derive(Clone, Copy)]
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
