//! Error types used in the library.
//!
//! - Most of these are very unlikely to occur during use.
//! - Some of these are internally expected but --- e.g. BCP errors are used to control the flow of a solve.
//! - Some are external --- e.g. a comtext may return an `AssumptionConflict` error to highlight a request to assume a literal would results in an unsatisfiable formula.
//! In this case information about satisfiability is obtained and the solver may (if satsfiable) continue to be used for further queries.
//!
//! Names of the error enums --- for the most part --- overlap with corresponding structs.
//  As such, throughout the library err::{self} is often used to prefix use of the types with `err::`.

use crate::db::keys::ClauseKey;

/// Noted errors during conflict analysis.
pub enum Analysis {
    /// Somehow resolution resolved to an empty clause.
    EmptyResolution,
    /// Resolution failed to terminate with an asserting clause.
    NoAssertion,
    /// Some issue with the resolution buffer.
    Buffer,
    /// Some issue with the clause store.
    ClauseDB,
    /// Resolution failed to stop at the required criteria.
    FailedStoppingCriteria,
}

/// Noted errors during boolean constraint propagation.
pub enum BCP {
    /// A conflict was found.
    /// This is expected from time to time, and a learning opportunity.
    Conflict(ClauseKey),
    /// Some corruption in the watched literals of a clause.
    /// This is unexpected.
    CorruptWatch,
}

#[derive(Debug)]
/// Noted errors when building a context.
///
/// These are general errors which wrap specific errors.
pub enum Build {
    /// A request to some other part of the context led to an error.
    Context(Context),
    /// An error while parsing.
    Parse(Parse),
    /// Interaction with a clause database led to an error.
    ClauseDB(ClauseDB),
}

/// An error in the clause database.
#[derive(Debug, Clone, Copy)]
pub enum ClauseDB {
    /// Attempt to get a unit clause by a key (the key is the literal)
    GetUnitKey,
    /// Attempt to transfer a unit clause.
    TransferUnit,
    /// Attempt to transfer a binary clause.
    TransferBinary,
    /// There was some issue with watches when transfering a clause.
    TransferWatch,
    /// A learnt cluase is missing.
    MissingLearned,
    /// An invalid key token.
    InvalidKeyToken,
    /// An invalid key index.
    InvalidKeyIndex,
    /// Some attempt was made to store an empty clause.
    EmptyClause,
    /// Some attempt was made to store a unit clause.
    UnitClause,
    /// All possible keys have been used for some clause type (formula/binary/long etc).
    StorageExhausted,
}

/// An error in the context.
#[derive(Debug, Clone, Copy)]
pub enum Context {
    /// Aka. an assumption was made after some choice.
    /// In principle, not an issue.
    /// Still, significant complexity is avoided by denying this possibility.
    AssumptionAfterChoice,
    /// An attempt to make an assumption which conflicts with some proven literal.
    AssumptionConflict,
    /// Somehow checking the literal returned that the literal was set.
    /// This is likely very puzzling.
    AssumptionSet,
    /// Failed to queue a literal.
    ///
    /// This is likely very puzzling.
    QueueConflict,
    /// The error from an interaction with the clause database.
    ///
    /// This is likely very puzzling.
    ClauseDB,
    /// Failed to backjump.
    ///
    /// This is likely very puzzling.
    Backjump,
    /// Analysis failed for some reason.
    Analysis,
    /// BCP failed for some reason.
    ///
    /// A full failure of conflict analysis which cannot be learnt from.
    BCP,
    Preprocessing,
}

/// An error during parsing.
#[derive(Debug)]
pub enum Parse {
    /// Some issue with the problem specification in a DIMACS input.
    ProblemSpecification,
    /// Some unspecific problem at a specific line.
    Line(usize),
    /// The problem specification of some DIMACS input is not in the header of the input.
    MisplacedProblem(usize),
    /// A negation character was read, but no candidate for negation was found.
    Negation,
    /// No file was found.
    NoFile,
}

pub enum Preprocessing {
    Pure,
}

pub enum Queue {
    Conflict,
}

pub enum Report {
    /// Failure to retreive a clause from the store for any reason.
    StoreFailure,
    /// An unsatisfiable core could be not constructed.
    /// Perhaps the clause was satisfiable?
    UnsatCoreUnavailable,
}

/// An error during resolution.
#[derive(Debug)]
pub enum ResolutionBuffer {
    /// A clause could not be found.
    LostClause,
    /// A minor headache… this can be disabled!
    Subsumption,
    /// Somehow the resolved clause is satisfied on the valuation used for assertion checking.
    /// This is quite serious, unless the wrong valuation has been used…
    SatisfiedClause,
    Transfer,
}

/// An error with the writer for FRAT proofs.
pub enum FRAT {
    /// A corrupt clause buffer.
    /// It is likely the addition of a clause was not noticed and the clause buffer was not cleared.
    CorruptClauseBuffer,
    /// A corrupt resolution buffer.
    /// It is likely the addition of a clause via resolution was not noticed and the clause buffer was not cleared.
    CorruptResolutionQ,
    /// Transfers are todo!
    TransfersAreTodo,
}

#[derive(Debug, Clone, Copy)]
pub enum Watch {
    /// A binary clause was found in a long watch list.
    /// Perhaps an issue during addition during addition or transfer of a clause…?
    NotLongInLong,
}

#[derive(Debug)]
pub enum Core {
    QueueMiss,
    EmptyBCPBuffer,
    CorruptClauseBuffer,
    MissedKey,
    NoConflict,
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
        Analysis::ClauseDB
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

impl From<ClauseDB> for Build {
    fn from(e: ClauseDB) -> Self {
        Self::ClauseDB(e)
    }
}

impl From<Context> for Build {
    fn from(e: Context) -> Self {
        Self::Context(e)
    }
}
