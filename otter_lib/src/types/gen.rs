use crate::{db::keys::ClauseKey, structures::literal::Literal};

pub enum Analysis {
    MissedImplication(ClauseKey, Literal),
    UnitClause(ClauseKey),
    FundamentalConflict,
    AssertingClause(ClauseKey, Literal),
}

pub enum Choice {
    Made,
    Exhausted,
}

pub enum Expansion {
    Conflict,
    UnitClause(ClauseKey),
    AssertingClause(ClauseKey, Literal),
    Exhausted,
}

pub enum Queue {
    Qd,
}

#[derive(Debug)]
pub enum RBuf {
    FirstUIP,
    Exhausted,
    Proof,
    Missed(ClauseKey, Literal),
}

#[derive(Debug, PartialEq, Eq)]
pub enum Solve {
    Initialised,
    AssertingClause,
    NoSolution,
    UnitClause,
    ChoiceMade,
    FullValuation,
}

pub enum Value {
    NotSet,
    Match,
    Conflict,
}

// The status of a watched literal
#[derive(Clone, Copy, PartialEq)]
pub enum Watch {
    Witness,  // The watched literal has a matching value.
    None,     // The watched literal has no value.
    Conflict, // watched literal has a conflicting value.
}

pub mod src {
    use super::*;

    #[derive(Clone, Copy, Debug)]
    pub enum Clause {
        Original,   // Read from a formula
        Resolution, // Derived via resolution (during analysis, etc.)
    }

    /// how a literal was settled
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    #[allow(clippy::upper_case_acronyms)]
    pub enum Literal {
        Choice,         // a choice made where the alternative may make a SAT difference
        BCP(ClauseKey), // direct from BCP
        Original,
    }
}
