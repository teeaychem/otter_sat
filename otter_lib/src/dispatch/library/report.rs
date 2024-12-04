use crate::{
    db::keys::ClauseKey,
    structures::{clause::Clause, literal::Literal},
};

#[derive(Clone)]
pub enum Report {
    Solve(self::Solve),
    ClauseDB(self::ClauseDB),
    Parser(self::Parser),
    LiteralDB(self::LiteralDB),
    Finish,
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum Solve {
    Satisfiable,
    Unsatisfiable,
    TimeUp,
    Unknown,
}

#[derive(PartialEq, Eq, Clone, Debug)]
pub enum ClauseDB {
    Active(ClauseKey, Clause),
    ActiveUnit(Literal),
}

#[derive(PartialEq, Eq, Clone, Debug)]
pub enum Parser {
    Load(String),
    Expected(usize, usize),
    Counts(usize, usize),
    ContextClauses(usize),
}

#[derive(PartialEq, Eq, Clone, Debug)]
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
            Self::Expected(v, c) => write!(f, "Expected:     {v} variables and {c} clauses"),
            Self::Counts(v, c) => write!(f, "Parse result: {v} variables and {c} clauses"),
            Self::ContextClauses(c) => write!(f, "{c} clauses are in the context"),
        }
    }
}
