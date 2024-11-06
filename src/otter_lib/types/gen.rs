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
    AssertingClause(ClauseKey),
    MissedImplication(ClauseKey),
    NoSolution(ClauseKey),
    Proof(ClauseKey),
    ChoiceMade,
    FullValuation,
    NoClauses,
}
