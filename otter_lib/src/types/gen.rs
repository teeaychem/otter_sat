use crate::{db::keys::ClauseKey, structures::literal::vbLiteral};

pub enum Analysis {
    MissedImplication(ClauseKey, vbLiteral),
    UnitClause(ClauseKey),
    FundamentalConflict,
    AssertingClause(ClauseKey, vbLiteral),
}

pub enum Choice {
    Made,
    Exhausted,
}

pub enum Expansion {
    Conflict,
    UnitClause(ClauseKey),
    AssertingClause(ClauseKey, vbLiteral),
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
    Missed(ClauseKey, vbLiteral),
}

#[allow(non_camel_case_types)]
#[derive(Debug, PartialEq, Eq)]
pub enum dbStatus {
    Consistent,
    Inconsistent,
    Unknown,
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
    }
}
