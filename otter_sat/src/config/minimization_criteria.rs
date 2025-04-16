use std::str::FromStr;

/// Variant minimization criterias to use during resolution-based analysis.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum MinimizationCriteria {
    /// Recursively examine the implication graph from BCP to determine whether each literal in a learnt clause would follow from the other literals and proven literals.
    RecursiveBCP = 0,

    /// Omit proven literals from learnt clauses.
    Proven,

    /// No clause minimization.
    None,
}

impl std::fmt::Display for MinimizationCriteria {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::RecursiveBCP => write!(f, "RecursiveBCP"),
            Self::Proven => write!(f, "Proven"),
            Self::None => write!(f, "None"),
        }
    }
}

impl MinimizationCriteria {
    /// The minimum MinimizationCriteria type.
    pub const MIN: MinimizationCriteria = MinimizationCriteria::None;

    /// The maximum MinimizationCriteria type.
    pub const MAX: MinimizationCriteria = MinimizationCriteria::RecursiveBCP;
}

impl FromStr for MinimizationCriteria {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "RecursiveBCP" => Ok(Self::RecursiveBCP),
            "Proven" => Ok(Self::Proven),
            "None" => Ok(Self::None),
            _ => Err(()),
        }
    }
}
