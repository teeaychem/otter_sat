use std::str::FromStr;

/// Variant stopping criterias to use during resolution-based analysis.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum StoppingCriteria {
    /// Stop at the first unique implication point.
    ///
    /// In other words, apply resolution until the clause obtained by resolution is asserting on the current valuation without the last decision made, and any consequences of that decision.
    FirstUIP = 0,

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

impl StoppingCriteria {
    /// The minimum StoppingCriteria type.
    pub const MIN: StoppingCriteria = StoppingCriteria::FirstUIP;

    /// The maximum StoppingCriteria type.
    pub const MAX: StoppingCriteria = StoppingCriteria::None;
}

impl FromStr for StoppingCriteria {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "FirstUIP" => Ok(Self::FirstUIP),

            "None" => Ok(Self::None),

            _unkown_string => Err(()),
        }
    }
}
