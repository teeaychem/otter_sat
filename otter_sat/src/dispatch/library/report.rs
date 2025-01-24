//! Details on the result of some procedure.
use crate::{
    db::ClauseKey,
    structures::{clause::cClause, literal::cLiteral},
};

/// Reports from the context.
#[derive(Clone)]
pub enum Report {
    /// Information regarding a solve.
    Solve(self::SolveReport),

    /// Information regarding the clause database.
    ClauseDB(self::ClauseDBReport),

    /// Information regarding the literal database.
    LiteralDB(self::LiteralDBReport),

    /// Information regarding the parse when building the context.
    Parser(self::ParserReport),

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

    /// Satisfiability of the formula of the context is unkown, for some reason.
    Unknown,
}

/// Information regarding the clause database.
// TODO: It would be nice to break down the dispatch of the clause in line with other dispatches.
#[derive(PartialEq, Eq, Clone)]
pub enum ClauseDBReport {
    /// An active non-unit clause.
    Active(ClauseKey, cClause),
    /// An active unit clause
    ActiveUnit(cLiteral),
}

/// Information regarding the parse when building the context.
#[derive(PartialEq, Eq, Clone, Debug)]
pub enum ParserReport {
    /// A DIMACS file has been loaded.
    Load(String),

    /// The expected clause/literal count based on the header of a DIMACS file.
    Expected(usize, usize),

    /// A count of clauses/literals from parsing a DIMACS file.
    Counts(usize, usize),

    /// The count of clauses added to the context from parsing a DIMACS file.
    ContextClauses(usize),
}

/// Information regarding the literal database.
#[derive(PartialEq, Eq, Clone)]
pub enum LiteralDBReport {}

impl std::fmt::Display for self::SolveReport {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Satisfiable => write!(f, "Satisfiable"),
            Self::Unsatisfiable => write!(f, "Unsatisfiable"),
            Self::TimeUp => write!(f, "Unkown"),
            Self::Unknown => write!(f, "Unkown"),
        }
    }
}

impl std::fmt::Display for self::ParserReport {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Load(formula) => write!(f, "Parsing \"{formula}\""),
            Self::Expected(a, c) => write!(f, "Expected:     {a} atoms and {c} clauses"),
            Self::Counts(a, c) => write!(f, "Parse result: {a} atoms and {c} clauses"),
            Self::ContextClauses(c) => write!(f, "{c} clauses are in the context"),
        }
    }
}
