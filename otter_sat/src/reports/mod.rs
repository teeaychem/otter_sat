/*!
Reports for the context.
*/

use crate::context::ContextState;

pub mod frat;

/// High-level reports regarding a solve.
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum Report {
    /// The formula of the context is satisfiable.
    Satisfiable,

    /// The formula of the context is unsatisfiable.
    Unsatisfiable,

    /// Satisfiability of the formula of the context is unknown, for some reason.
    Unknown,
}

impl From<ContextState> for Report {
    fn from(value: ContextState) -> Self {
        match value {
            ContextState::Configuration | ContextState::Input | ContextState::Solving => {
                Self::Unknown
            }
            ContextState::Satisfiable => Self::Satisfiable,
            ContextState::Unsatisfiable(_) => Self::Unsatisfiable,
        }
    }
}

impl std::fmt::Display for Report {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Satisfiable => write!(f, "Satisfiable"),
            Self::Unsatisfiable => write!(f, "Unsatisfiable"),
            Self::Unknown => write!(f, "Unknown"),
        }
    }
}
