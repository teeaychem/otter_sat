use crate::structures::literal::Literal;

use crate::context::stores::ClauseKey;

pub enum Dispatch {
    // δ
    ClauseDB(delta::ClauseDB),
    Level(delta::Level),
    Parser(delta::Parser),
    Resolution(delta::Resolution),
    VariableDB(delta::Variable),
    // misc
    SolveComment(comment::Solve),
    SolveReport(report::Solve),
    ClauseDBReport(report::ClauseDB),
    VariableDBReport(report::VariableDB),
}

impl std::fmt::Display for comment::Solve {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::AllTautological => write!(f, "All clauses of the formula are tautological"),
            Self::FoundEmptyClause => write!(f, "The formula contains an empty clause"),
            Self::NoClauses => write!(f, "The formula does not contain any clauses"),
            Self::TimeUp => write!(f, "Time limit exceeded"),
        }
    }
}

pub mod report {
    use super::*;

    #[derive(PartialEq, Eq, Clone, Copy, Debug)]
    pub enum Solve {
        Satisfiable,
        Unsatisfiable,
        Unknown,
    }

    #[derive(PartialEq, Eq, Clone, Debug)]
    pub enum ClauseDB {
        Active(ClauseKey, Vec<Literal>),
    }

    #[derive(PartialEq, Eq, Clone, Debug)]
    pub enum VariableDB {
        Active(Literal),
    }
}

pub mod delta {
    use super::*;

    pub enum Variable {
        Internalised(String, u32),
        Falsum(Literal),
    }

    pub enum ClauseBuider {
        Start,
        Index(u32),
        Literal(Literal),
        End,
    }

    pub enum ClauseDB {
        TransferBinary(ClauseKey, ClauseKey, Vec<Literal>),
        Deletion(ClauseKey, Vec<Literal>),
        BinaryFormula(ClauseKey, Vec<Literal>),
        BinaryResolution(ClauseKey, Vec<Literal>),
        Formula(ClauseKey, Vec<Literal>),
        Learned(ClauseKey, Vec<Literal>),
    }

    pub enum Parser {
        Load(String),
        Expected(usize, usize),
        Complete(usize, usize),
        ContextClauses(usize),
    }

    pub enum Resolution {
        Start,
        Finish,
        Used(ClauseKey),
        Subsumed(ClauseKey, Literal),
    }

    #[derive(Debug)]
    pub enum Level {
        Assumption(Literal),
        ResolutionProof(Literal),
        BCP(Literal),
        Pure(Literal),
    }
}

pub mod comment {
    pub enum Solve {
        AllTautological,
        FoundEmptyClause,
        NoClauses, // "c The formula contains no clause and so is interpreted as ⊤
        TimeUp,
    }
}

impl std::fmt::Display for report::Solve {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Satisfiable => write!(f, "Satisfiable"),
            Self::Unsatisfiable => write!(f, "Unsatisfiable"),
            Self::Unknown => write!(f, "Unkown"),
        }
    }
}

impl std::fmt::Display for delta::Parser {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Load(formula) => write!(f, "Parsing \"{formula}\""),
            Self::Expected(v, c) => write!(f, "Expected:     {v} variables and {c} clauses"),
            Self::Complete(v, c) => write!(f, "Parse result: {v} variables and {c} clauses"),
            Self::ContextClauses(c) => write!(f, "{c} clauses are in the context"),
        }
    }
}
