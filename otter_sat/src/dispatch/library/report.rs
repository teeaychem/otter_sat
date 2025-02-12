/*!
Details on the result of some procedure.
*/
use crate::{
    db::ClauseKey,
    structures::{clause::CClause, literal::CLiteral},
};

/// Reports from the context.
#[derive(Clone)]
pub enum Report {
    /// Information regarding a solve.
    Solve(self::SolveReport),

    /// Information regarding the clause database.
    ClauseDB(self::ClauseDBReport),

    /// No further dispatches will be sent regarding the current solve.
    Finish,
}

/// High-level reports regarding a solve.
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum SolveReport {
    /// The formula of the context is satisfiable.
    Satisfiable,

    /// The formula of the context is unsatisfiable.
    Unsatisfiable,

    /// Satisfiability of the formula of the context could not be determined within the time allowed.
    TimeUp,

    /// Satisfiability of the formula of the context is unknown, for some reason.
    Unknown,
}

/// Information regarding the clause database.
// TODO: It would be nice to break down the dispatch of the clause in line with other dispatches.
#[derive(PartialEq, Eq, Clone)]
pub enum ClauseDBReport {
    /// An active non-unit clause.
    Active(ClauseKey, CClause),

    /// An active unit clause
    ActiveOriginalUnit(CLiteral),

    ActiveAdditionUnit(CLiteral),
}

/// Information regarding the literal database.
#[derive(PartialEq, Eq, Clone)]
pub enum LiteralDBReport {}

impl std::fmt::Display for self::SolveReport {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Satisfiable => write!(f, "Satisfiable"),
            Self::Unsatisfiable => write!(f, "Unsatisfiable"),
            Self::TimeUp => write!(f, "Unknown"),
            Self::Unknown => write!(f, "Unknown"),
        }
    }
}
