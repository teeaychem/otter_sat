/*!
Error types used in the library.

- Most of these are very unlikely to occur during use.
- Some of these are internally expected --- e.g. BCP errors are used to control the flow of a solve.
- Others are external --- e.g. a comtext may return an `AssumptionConflict` error to highlight a request to assume a literal would results in an unsatisfiable formula.
  In this case information about satisfiability is obtained and the solver may (if satsfiable) continue to be used for further queries.

Names of the error enums --- for the most part --- overlap with corresponding structs.
*/
//  As such, throughout the library err::{self} is often used to prefix use of the types with `err::`.

use crate::{db::ClauseKey, structures::literal::CLiteral};

/// A union of varied error kinds.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ErrorKind {
    /// An error during conflict analysis.
    Analysis(AnalysisError),

    /// An error when building a context.
    Build(BuildError),

    /// An error in the clause database.
    ClauseDB(ClauseDBError),

    /// An error in the atom database.
    AtomDB(AtomDBError),

    /// An related to parsing.
    Parse(ParseError),

    /// An related to preprocessing.
    Preprocessing(PreprocessingError),

    /// An related to BCP.
    BCP(BCPError),

    /// An related to the resolution buffer.
    ResolutionBuffer(ResolutionBufferError),

    /// An related to the state of the context.
    State(StateError),

    /// An related to backjumping.
    Backjump,

    /// The attempted action could not be completed given the state of the context.
    InvalidState,

    /// The attempted action could not be completed given the valuation of the context.
    ValuationConflict,

    /// The attempted action could not be completed given the valuation of the context, specifically due to the noted literal.
    SpecificValuationConflict(CLiteral),

    /// The assumption conflicts with a proven literal.
    AssumptionConflict(CLiteral),

    /// A fundamental conflict has been observed (and recorded)
    FundamentalConflict,
}

/// An error during conflict analysis.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AnalysisError {
    /// Somehow resolution resolved to an empty clause.
    EmptyResolution,

    /// Resolution failed to terminate with an asserting clause.
    NoAssertion,

    /// Resolution failed to stop at the required criteria.
    FailedStoppingCriteria,
}

impl From<AnalysisError> for ErrorKind {
    fn from(e: AnalysisError) -> Self {
        ErrorKind::Analysis(e)
    }
}

/// An error from the clause database.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AtomDBError {
    /// There are no more fresh atoms.
    AtomsExhausted,
}

impl From<AtomDBError> for ErrorKind {
    fn from(e: AtomDBError) -> Self {
        ErrorKind::AtomDB(e)
    }
}

/// Noted errors during boolean constraint propagation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BCPError {
    /// A conflict was found.
    /// This is expected from time to time, and a learning opportunity.
    Conflict(ClauseKey),

    /// Some corruption in the watched literals of a clause.
    /// This is unexpected.
    CorruptWatch,
}

impl From<BCPError> for ErrorKind {
    fn from(e: BCPError) -> Self {
        ErrorKind::BCP(e)
    }
}

/// Noted errors when building a context.
///
/// These are general errors which wrap specific errors.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BuildError {
    /// An clear instance of an unsatisfiable clause
    Unsatisfiable,
}

impl From<BuildError> for ErrorKind {
    fn from(e: BuildError) -> Self {
        ErrorKind::Build(e)
    }
}

/// Errors in the clause database.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ClauseDBError {
    /// Attempt to get a unit clause by a key (the key is the literal)
    GetOriginalUnitKey,

    /// Attempt to transfer a unit clause.
    TransferUnit,

    /// Attempt to transfer a binary clause.
    TransferBinary,

    /// A unit or binary clause was found in a long watch list.
    /// Perhaps an issue during addition during addition or transfer of a clause…?
    CorruptList,

    /// A learnt cluase is missing.
    Missing,

    /// An invalid key token.
    InvalidKeyToken,

    /// An invalid key index.
    InvalidKeyIndex,

    /// Some attempt was made to store an empty clause.
    EmptyClause,

    /// All possible keys have been used for some clause type (formula/binary/long etc).
    StorageExhausted,

    /// A unit clause was added after some decision has been made.
    ///
    /// Ideally, this case could be handled and this error removed.
    DecisionMade,
}

impl From<ClauseDBError> for ErrorKind {
    fn from(e: ClauseDBError) -> Self {
        ErrorKind::ClauseDB(e)
    }
}

/// Errors with the writer for FRAT proofs.
pub enum FRATError {
    /// A corrupt clause buffer.
    /// It is likely the addition of a clause was not noticed and the clause buffer was not cleared.
    CorruptClauseBuffer,

    /// A corrupt resolution buffer.
    /// It is likely the addition of a clause via resolution was not noticed and the clause buffer was not cleared.
    CorruptResolutionQ,

    /// Transfers are todo!
    TransfersAreTodo,
}

/// Errors during parsing.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ParseError {
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

    /// A required delimiter was not found.
    MissingDelimiter,
}

impl From<ParseError> for ErrorKind {
    fn from(e: ParseError) -> Self {
        ErrorKind::Parse(e)
    }
}

/// An errror during preprocessing.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PreprocessingError {
    /// The formula was identified as unsatisfiable.
    Unsatisfiable,
}

impl From<PreprocessingError> for ErrorKind {
    fn from(e: PreprocessingError) -> Self {
        ErrorKind::Preprocessing(e)
    }
}

/// Errors during resolution.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ResolutionBufferError {
    /// A clause could not be found.
    LostClause,

    /// A minor headache… this can be disabled!
    Subsumption(SubsumptionError),

    /// Somehow the resolved clause is satisfied on the valuation used for assertion checking.
    /// This is quite serious, unless the wrong valuation has been used…
    SatisfiedClause,

    /// A key failed to unlock a clause.
    MissingClause,

    /// The trail was exhausted without finding a unique implication point.
    Exhausted,
}

impl From<ResolutionBufferError> for ErrorKind {
    fn from(e: ResolutionBufferError) -> Self {
        ErrorKind::ResolutionBuffer(e)
    }
}

/// Errors due to the state of the solver.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StateError {
    /// A solve is in progress.
    SolveInProgress,
}

/// Errors during [subsumption](crate::db::clause::db_clause::dbClause::subsume).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SubsumptionError {
    /// The clause was too short for subsumption to.
    ShortClause,

    /// The pivot for subsumption was not found.
    NoPivot,

    /// Watches for the subsumed atom were not found.
    WatchError,

    /// Unable to retreive a clause from the clause database.
    ClauseDB,
}

impl From<SubsumptionError> for ResolutionBufferError {
    fn from(e: SubsumptionError) -> Self {
        ResolutionBufferError::Subsumption(e)
    }
}
