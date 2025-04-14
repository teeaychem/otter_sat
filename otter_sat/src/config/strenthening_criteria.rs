use std::str::FromStr;

/// Variant stregnthening criterias to use during resolution-based analysis.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum StrengtheningCriteria {
    /// Recursively examine the implication graph from BCP to determine whether each literal in a learnt clause would follow from the other literals and proven literals.
    RecursiveBCP = 0,

    /// Do not apply strenthening (other than omitting proven literals).
    None,
}

impl std::fmt::Display for StrengtheningCriteria {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::RecursiveBCP => write!(f, "RecursiveBCP"),
            Self::None => write!(f, "None"),
        }
    }
}

impl StrengtheningCriteria {
    /// The minimum StoppingCriteria type.
    pub const MIN: StrengtheningCriteria = StrengtheningCriteria::None;

    /// The maximum StrengtheningCriteria type.
    pub const MAX: StrengtheningCriteria = StrengtheningCriteria::RecursiveBCP;
}

impl FromStr for StrengtheningCriteria {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "RecursiveBCP" => Ok(Self::RecursiveBCP),
            "None" => Ok(Self::None),
            _ => Err(()),
        }
    }
}
