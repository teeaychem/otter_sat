use crate::context::stores::ClauseKey;

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum Report {
    Satisfiable,
    Unsatisfiable,
    Unknown,
}

#[derive(Debug, PartialEq, Eq)]
pub enum SolveStatus {
    Initialised,
    AssertingClause,
    MissedImplication,
    NoSolution,
    Proof,
    ChoiceMade,
    FullValuation,
    NoClauses,
}
