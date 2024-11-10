use crate::context::stores::ClauseKey;

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
