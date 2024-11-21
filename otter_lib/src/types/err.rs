/*
Names of the error enums --- for the most part --- overlap with their corresponding enums

So, intended use is to namespace errors via the module.

For example:
- use err::{self}
- …
- err::<TYPE>


 */

use crate::db::keys::ClauseKey;

pub enum Analysis {
    ResolutionNotStored,    // For some reason the resolved clause was not stored
    EmptyResolution,        // Somehow resolution resolved to an empty clause
    NoAssertion,            // Resolution failed to terminate with an asserting clause
    Buffer,                 // Some issue with the resolution buffer
    ClauseStore,            // Some issue with the clause store
    FailedStoppingCriteria, // Resolution failed to stop at the required criteria
}

pub enum BCP {
    Conflict(ClauseKey),
    CorruptWatch,
}

#[derive(Debug)]
pub enum Build {
    UnitClauseConflict,
    Conflict,
    Parse(Parse),
    ClauseStore(ClauseDB),
}

#[derive(Debug, Clone, Copy)]
pub enum ClauseDB {
    TransferBinary,   // Attempt to transfer a binary clause
    TransferWatch,    // There was some issue with watches when transfering a clause
    MissingLearned,   // A learnt cluase is missing
    InvalidKeyToken,  // An invalid key token
    InvalidKeyIndex,  // An invalid key index
    EmptyClause,      // An attempt was made to store an empty clause
    UnitClause,       // An attempt was made to store a unit clause
    StorageExhausted, // All possible keys have been used for some clause type (formula/binary/long etc)
}

#[derive(Debug, Clone, Copy)]
pub enum Context {
    AssumptionAfterChoice, // Aka. an assumption not made on level zero
    AssumptionConflict, // An attempt to make an assumption which conflicts with some proven literal
    AssumptionSet,      // Somehow checking the literal returned that the literal was set.
    QueueConflict,      // Failed to queue a literal
    ClauseDB,           // The error from an interaction with the clause database
    Backjump,           // Failed to backjump
    Analysis,           // Analysis failed for some reason
    BCP,                // BCP failed for some reason
    Preprocessing,
}

#[derive(Debug)]
pub enum Parse {
    ProblemSpecification,
    Line(usize),
    MisplacedProblem(usize),
    NoVariable,
    NoFile,
}

pub enum Preprocessing {
    Pure,
}

pub enum Queue {
    Conflict,
}

pub enum Report {
    StoreFailure, // Failure to retreive a clause from the store for any reason
    UnsatCoreUnavailable,
}

#[derive(Debug)]
pub enum RBuf {
    LostClause,
    Subsumption,
    SatisfiedResolution,
    Transfer,
}

#[derive(Debug, Clone, Copy)]
pub enum Watch {
    BinaryInLong, // Found a binary clause in a long watch list
}

// Ignore the reason for failing to transfer a clause
impl From<Watch> for ClauseDB {
    fn from(_: Watch) -> Self {
        ClauseDB::TransferWatch
    }
}

// Ignore the reason for failing to retreive a clause
impl From<ClauseDB> for Analysis {
    fn from(_: ClauseDB) -> Self {
        Analysis::ClauseStore
    }
}

// Ignore the reason for failing to retreive a clause
impl From<ClauseDB> for Report {
    fn from(_: ClauseDB) -> Self {
        Report::StoreFailure
    }
}

// Ignore the reason for failing to retreive a clause
impl From<Queue> for Context {
    fn from(_: Queue) -> Self {
        Self::QueueConflict
    }
}

impl From<ClauseDB> for Context {
    fn from(_: ClauseDB) -> Self {
        Context::ClauseDB
    }
}

impl From<Analysis> for Context {
    fn from(_: Analysis) -> Self {
        Context::Analysis
    }
}

impl From<Preprocessing> for Context {
    fn from(_: Preprocessing) -> Self {
        Self::Preprocessing
    }
}
