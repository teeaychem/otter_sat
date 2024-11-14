use crate::db::keys::ClauseKey;

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

#[derive(Debug, Clone, Copy)]
pub enum Step {
    Conflict,
    ChoicesExhausted,
    ChoiceMade,
    One,
}

pub enum QStatus {
    Qd,
}

/// how a literal was settled
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[allow(clippy::upper_case_acronyms)]
pub enum LiteralSource {
    Choice,                // a choice made where the alternative may make a SAT difference
    Pure,                  // a choice made when the alternative would make no SAT difference
    Analysis(ClauseKey),   // the literal must be the case for SAT given some valuation
    Resolution(ClauseKey), // there was no reason to store the resolved clause
    BCP(ClauseKey),
    Missed(ClauseKey),
    Assumption,
}

pub enum Value {
    NotSet,
    Match,
    Conflict,
}
