//! Details on the result of some procedure.
use crate::{
    db::ClauseKey,
    structures::{clause::vClause, literal::abLiteral},
};

/// Reports from the context.
#[derive(Clone)]
pub enum Report {
    /// Information regarding a solve.
    Solve(self::Solve),

    /// Information regarding the clause database.
    ClauseDB(self::ClauseDB),

    /// Information regarding the literal database.
    LiteralDB(self::LiteralDB),

    /// Information regarding the parse when building the context.
    Parser(self::Parser),

    /// No further dispatches will be sent regarding the current solve.
    Finish,
}

/// High-level reports regarding a solve.
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum Solve {
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
pub enum ClauseDB {
    /// An active non-unit clause.
    Active(ClauseKey, vClause),
    /// An active unit clause
    ActiveUnit(abLiteral),
}

/// Information regarding the parse when building the context.
#[derive(PartialEq, Eq, Clone, Debug)]
pub enum Parser {
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
pub enum LiteralDB {}

impl std::fmt::Display for self::Solve {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Satisfiable => write!(f, "Satisfiable"),
            Self::Unsatisfiable => write!(f, "Unsatisfiable"),
            Self::TimeUp => write!(f, "Unkown"),
            Self::Unknown => write!(f, "Unkown"),
        }
    }
}

impl std::fmt::Display for self::Parser {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Load(formula) => write!(f, "Parsing \"{formula}\""),
            Self::Expected(a, c) => write!(f, "Expected:     {a} atoms and {c} clauses"),
            Self::Counts(a, c) => write!(f, "Parse result: {a} atoms and {c} clauses"),
            Self::ContextClauses(c) => write!(f, "{c} clauses are in the context"),
        }
    }
}
