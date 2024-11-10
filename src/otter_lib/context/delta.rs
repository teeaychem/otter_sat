use crate::structures::literal::Literal;

pub enum Delta {
    ClauseBuilder(ClauseBuider),
    SolveComment(SolveComment),
    SolveReport(SolveReport),
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
}

impl std::fmt::Display for SolveComment {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::AllTautological => write!(f, "All clauses of the formula are tautological"),
            Self::FoundEmptyClause => write!(f, "The formula contains an empty clause"),
            Self::NoClauses => write!(f, "The formula does not contain any clauses"),
        }
    }
}

pub enum SolveReport {
    Satisfiable,
    Unsatisfiable,
    Unkown,
}

impl std::fmt::Display for SolveReport {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Satisfiable => write!(f, "Satisfiable"),
            Self::Unsatisfiable => write!(f, "Unsatisfiable"),
            Self::Unkown => write!(f, "Unkown"),
        }
    }
}
