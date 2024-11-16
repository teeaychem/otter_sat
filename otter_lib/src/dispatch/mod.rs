use crate::{context::Context, db::keys::ClauseKey, structures::literal::Literal};

pub mod frat;
pub mod receivers;
pub mod transmitters;

#[derive(Clone)]
pub enum Dispatch {
    // δ
    ClauseDB(delta::ClauseDB),
    Level(delta::Level),
    Parser(delta::Parser),
    Resolution(delta::Resolution),
    VariableDB(delta::Variable),
    BCP(delta::BCP),
    // misc
    SolveComment(comment::Solve),
    SolveReport(report::Solve),
    ClauseDBReport(report::ClauseDB),
    VariableDBReport(report::VariableDB),
    Finish,
    // stats
    Stats(stat::Count),
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

pub mod stat {
    use std::time::Duration;

    #[derive(Clone)]
    pub enum Count {
        ICD(usize, usize, usize),
        Time(Duration),
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

    #[derive(Clone)]
    pub enum BCP {
        Instance(Literal, ClauseKey, Literal), // Literal (left) + ClauseKey -> Literal (right)
        Conflict(Literal, ClauseKey),          // Literal + ClauseKey -> falsum
    }

    #[derive(Clone)]
    pub enum Variable {
        Internalised(String, u32),
        Unsatisfiable(ClauseKey),
    }

    #[derive(Clone)]
    pub enum ClauseBuider {
        Start,
        Index(u32),
        Literal(Literal),
        End,
    }

    #[derive(Clone)]
    pub enum ClauseDB {
        TransferBinary(ClauseKey, ClauseKey, Vec<Literal>),
        Deletion(ClauseKey, Vec<Literal>),
        BinaryOriginal(ClauseKey, Vec<Literal>),
        BinaryResolution(ClauseKey, Vec<Literal>),
        Original(ClauseKey, Vec<Literal>),
        Learned(ClauseKey, Vec<Literal>),
    }

    #[derive(Clone)]
    pub enum Parser {
        Load(String),
        Expected(usize, usize),
        Complete(usize, usize),
        ContextClauses(usize),
    }

    #[derive(Clone)]
    pub enum Resolution {
        Begin,
        End,
        Used(ClauseKey),
        Subsumed(ClauseKey, Literal),
    }

    #[derive(Debug, Clone)]
    pub enum Level {
        Assumption(Literal),
        ResolutionProof(Literal),
        Proof(Literal),
        Forced(ClauseKey, Literal),
        Pure(Literal),
    }
}

pub mod comment {

    #[derive(Clone)]
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

impl Context {
    pub fn dispatch_active(&self) {
        self.clause_db.dispatch_active();

        for literal in self.literal_db.proven_literals() {
            let report = report::VariableDB::Active(*literal);
            self.tx.send(Dispatch::VariableDBReport(report));
        }
    }
}
