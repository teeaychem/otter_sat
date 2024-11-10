use crate::structures::literal::Literal;

use super::stores::ClauseKey;

pub enum Dispatch {
    ClauseDelta(ClauseBuider),
    ClauseStore(ClauseStoreDelta),
    SolveComment(SolveComment),
    SolveReport(SolveReport),
    Parser(Parser),
    Resolution(ResolutionDelta),
    Level(LevelDelta),
}

pub enum ClauseBuider {
    Start,
    Index(u32),
    Literal(Literal),
    End,
}

pub enum SolveComment {
    AllTautological,
    FoundEmptyClause,
    NoClauses, // "c The formula contains no clause and so is interpreted as ⊤
    TimeUp,
}

impl std::fmt::Display for SolveComment {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::AllTautological => write!(f, "All clauses of the formula are tautological"),
            Self::FoundEmptyClause => write!(f, "The formula contains an empty clause"),
            Self::NoClauses => write!(f, "The formula does not contain any clauses"),
            Self::TimeUp => write!(f, "Time limit exceeded"),
        }
    }
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum SolveReport {
    Satisfiable,
    Unsatisfiable,
    Unknown,
}

impl std::fmt::Display for SolveReport {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Satisfiable => write!(f, "Satisfiable"),
            Self::Unsatisfiable => write!(f, "Unsatisfiable"),
            Self::Unknown => write!(f, "Unkown"),
        }
    }
}

pub enum Parser {
    Processing(String),
    Expectation(usize, usize),
    Complete(usize, usize),
    ContextClauses(usize),
}

impl std::fmt::Display for Parser {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Processing(formula) => write!(f, "Parsing \"{formula}\""),
            Self::Expectation(v, c) => {
                write!(f, "Expectation is to get {v} variables and {c} clauses")
            }
            Self::Complete(v, c) => {
                write!(f, "Parsing complete with {v} variables and {c} clauses")
            }
            Parser::ContextClauses(c) => write!(f, "{c} clauses were added to the context"),
        }
    }
}

pub enum ClauseStoreDelta {
    TransferFormula(ClauseKey, ClauseKey),
    TransferLearned(ClauseKey, ClauseKey),
    Deletion(ClauseKey),
    BinaryFormula(ClauseKey, Vec<Literal>),
    BinaryResolution(ClauseKey, Vec<Literal>),
    Formula(ClauseKey, Vec<Literal>),
    Learned(ClauseKey, Vec<Literal>),
}

#[derive(Debug)]
pub enum LevelDelta {
    FormulaAssumption(Literal),
    ResolutionProof(Literal),
}

pub enum ResolutionDelta {
    Start,
    Finish,
    Used(ClauseKey),
}
