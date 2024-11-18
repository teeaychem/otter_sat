#[derive(Clone)]
pub enum Comment {
    Solve(self::Solve),
}

#[derive(Clone)]
pub enum Solve {
    AllTautological,
    FoundEmptyClause,
    NoClauses, // "c The formula contains no clause and so is interpreted as ⊤
    TimeUp,
}

impl std::fmt::Display for self::Solve {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::AllTautological => write!(f, "All clauses of the formula are tautological"),
            Self::FoundEmptyClause => write!(f, "The formula contains an empty clause"),
            Self::NoClauses => write!(f, "The formula does not contain any clauses"),
            Self::TimeUp => write!(f, "Time limit exceeded"),
        }
    }
}
