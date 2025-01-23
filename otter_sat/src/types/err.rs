//! Error types used in the library.
//!
//! - Most of these are very unlikely to occur during use.
//! - Some of these are internally expected but --- e.g. BCP errors are used to control the flow of a solve.
//! - Some are external --- e.g. a comtext may return an `AssumptionConflict` error to highlight a request to assume a literal would results in an unsatisfiable formula.
//!   In this case information about satisfiability is obtained and the solver may (if satsfiable) continue to be used for further queries.
//!
//! Names of the error enums --- for the most part --- overlap with corresponding structs.
//  As such, throughout the library err::{self} is often used to prefix use of the types with `err::`.

use crate::db::ClauseKey;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ErrorKind {
    Analysis(AnalysisErrorKind),
    Build(BuildErrorKind),
    ClauseDB(ClauseDBErrorKind),
    AtomDB(AtomDBErrorKind),
    Parse(ParseErrorKind),
}

/// Noted errors during conflict analysis.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AnalysisErrorKind {
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

impl From<AnalysisErrorKind> for ErrorKind {
    fn from(e: AnalysisErrorKind) -> Self {
        ErrorKind::Analysis(e)
    }
}

/// Noted errors during boolean constraint propagation.
pub enum BCPErrorKind {
    /// A conflict was found.
    /// This is expected from time to time, and a learning opportunity.
    Conflict(ClauseKey),
    /// Some corruption in the watched literals of a clause.
    /// This is unexpected.
    CorruptWatch,
}

/// Noted errors when building a context.
///
/// These are general errors which wrap specific errors.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BuildErrorKind {
    /// An clear instance of an unsatisfiable clause
    Unsatisfiable,
}

impl From<BuildErrorKind> for ErrorKind {
    fn from(e: BuildErrorKind) -> Self {
        ErrorKind::Build(e)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AtomDBErrorKind {
    /// There are no more fresh atoms.
    AtomsExhausted,
}

impl From<AtomDBErrorKind> for ErrorKind {
    fn from(e: AtomDBErrorKind) -> Self {
        ErrorKind::AtomDB(e)
    }
}

/// Errors in the clause database.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ClauseDBErrorKind {
    /// Attempt to get a unit clause by a key (the key is the literal)
    GetUnitKey,

    /// Attempt to transfer a unit clause.
    TransferUnit,

    /// Attempt to transfer a binary clause.
    TransferBinary,

    /// There was some issue with watches when transfering a clause.
    TransferWatch,

    /// A learnt cluase is missing.
    Missing,

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

    /// A unit clause was added after some decision has been made.
    ///
    /// Ideally, this case could be handled and this error removed.
    AddedUnitAfterDecision,

    /// An immediate conflict.
    ImmediateConflict,

    /// The clause conflicts with the current valuation.
    ///
    /// For example, due to assumption made.
    ValuationConflict,
}

impl From<ClauseDBErrorKind> for ErrorKind {
    fn from(e: ClauseDBErrorKind) -> Self {
        ErrorKind::ClauseDB(e)
    }
}

#[derive(Clone, Copy)]
pub enum SubsumptionErrorKind {
    ShortClause,
    NoPivot,
    WatchError,
    TransferFailure,
    ClauseTooShort,
    ClauseDB,
}

/// Errors in the context.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ContextErrorKind {
    /// Aka. an assumption was made after some decision.
    /// In principle, not an issue.
    /// Still, significant complexity is avoided by denying this possibility.
    AssumptionAfterDecision,
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

/// Errors during parsing.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ParseErrorKind {
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
    /// An empty string, where some non-empty string was required.
    Empty,
}

impl From<ParseErrorKind> for ErrorKind {
    fn from(e: ParseErrorKind) -> Self {
        ErrorKind::Parse(e)
    }
}

pub enum PreprocessingErrorKind {
    Pure,
}

pub enum ConsequenceQueueErrorKind {
    Conflict,
}

pub enum ReportErrorKind {
    /// Failure to retreive a clause from the store for any reason.
    StoreFailure,
    /// An unsatisfiable core could be not constructed.
    /// Perhaps the clause was satisfiable?
    UnsatCoreUnavailable,
}

/// Errors during resolution.
pub enum ResolutionBufferErrorKind {
    /// A clause could not be found.
    LostClause,
    /// A minor headache… this can be disabled!
    Subsumption,
    /// Somehow the resolved clause is satisfied on the valuation used for assertion checking.
    /// This is quite serious, unless the wrong valuation has been used…
    SatisfiedClause,
    Transfer,
    MissingClause,
}

/// Errors with the writer for FRAT proofs.
pub enum FRATErrorKind {
    /// A corrupt clause buffer.
    /// It is likely the addition of a clause was not noticed and the clause buffer was not cleared.
    CorruptClauseBuffer,
    /// A corrupt resolution buffer.
    /// It is likely the addition of a clause via resolution was not noticed and the clause buffer was not cleared.
    CorruptResolutionQ,
    /// Transfers are todo!
    TransfersAreTodo,
}

#[derive(Clone, Copy)]
pub enum WatchErrorKind {
    /// A binary clause was found in a long watch list.
    /// Perhaps an issue during addition during addition or transfer of a clause…?
    NotLongInLong,
}

#[derive(Debug)]
pub enum CoreErrorKind {
    QueueMiss,
    EmptyBCPBuffer,
    CorruptClauseBuffer,
    MissedKey,
    NoConflict,
}

// Ignore the reason for failing to transfer a clause
impl From<WatchErrorKind> for ClauseDBErrorKind {
    fn from(_: WatchErrorKind) -> Self {
        ClauseDBErrorKind::TransferWatch
    }
}

// Ignore the reason for failing to retreive a clause
impl From<ClauseDBErrorKind> for AnalysisErrorKind {
    fn from(_: ClauseDBErrorKind) -> Self {
        AnalysisErrorKind::ClauseDB
    }
}

// Ignore the reason for failing to retreive a clause
impl From<ClauseDBErrorKind> for ReportErrorKind {
    fn from(_: ClauseDBErrorKind) -> Self {
        ReportErrorKind::StoreFailure
    }
}

// Ignore the reason for failing to retreive a clause
impl From<ConsequenceQueueErrorKind> for ContextErrorKind {
    fn from(_: ConsequenceQueueErrorKind) -> Self {
        Self::QueueConflict
    }
}

impl From<ClauseDBErrorKind> for ContextErrorKind {
    fn from(_: ClauseDBErrorKind) -> Self {
        ContextErrorKind::ClauseDB
    }
}

impl From<AnalysisErrorKind> for ContextErrorKind {
    fn from(_: AnalysisErrorKind) -> Self {
        ContextErrorKind::Analysis
    }
}

impl From<PreprocessingErrorKind> for ContextErrorKind {
    fn from(_: PreprocessingErrorKind) -> Self {
        Self::Preprocessing
    }
}

impl From<SubsumptionErrorKind> for ResolutionBufferErrorKind {
    fn from(_value: SubsumptionErrorKind) -> Self {
        Self::Subsumption
    }
}
