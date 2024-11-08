use crate::context::stores::ClauseKey;

#[derive(Debug, Clone, Copy)]
pub enum ContextErr {
    AssumptionAfterChoice, // Aka. an assumption not made on level zero
    AssumptionConflict, // An attempt to make an assumption which conflicts with some proven literal
    AssumptionSet,      // Somehow checking the literal returned that the literal was set.
}

#[derive(Debug, Clone, Copy)]
pub enum StepErr {
    QueueConflict(ClauseKey), // Failed to queue a literal asserted by a conflict
    QueueProof(ClauseKey),    // Failed to queue a proven literal
    Backfall,                 // Faile to backjump
    AnalysisFailure,          // Analysis failed for some reason
    ChoiceFailure,            // Choice failed for some reason
    BCPFailure,               // BCP failed for some reason
    ClauseStore(ClauseStoreErr), // The error from an interaction with the clause store
}

// Cast a clause store error as a step error
impl From<ClauseStoreErr> for StepErr {
    fn from(value: ClauseStoreErr) -> Self {
        StepErr::ClauseStore(value)
    }
}

#[derive(Debug, Clone, Copy)]
pub enum ClauseStoreErr {
    TransferBinary,   // Attempt to transfer a binary clause
    TransferWatch,    // There was some issue with watches when transfering a clause
    MissingLearned,   // A learnt cluase is missing
    InvalidKeyToken,  // An invalid key token
    InvalidKeyIndex,  // An invalid key index
    EmptyClause,      // An attempt was made to store an empty clause
    UnitClause,       // An attempt was made to store a unit clause
    StorageExhausted, // All possible keys have been used for some clause type (formula/binary/long etc)
}

// Ignore the reason for failing to transfer a clause
impl From<WatchError> for ClauseStoreErr {
    fn from(_: WatchError) -> Self {
        ClauseStoreErr::TransferWatch
    }
}

#[derive(Debug, Clone, Copy)]
pub enum WatchError {
    BinaryInLong, // Found a binary clause in a long watch list
}

pub enum AnalysisError {
    ResolutionNotStored,    // For some reason the resolved clause was not stored
    EmptyResolution,        // Somehow resolution resolved to an empty clause
    NoAssertion,            // Resolution failed to terminate with an asserting clause
    Buffer,                 // Some issue with the resolution buffer
    ClauseStore,            // Some issue with the clause store
    FailedStoppingCriteria, // Resolution failed to stop at the required criteria
}

// Ignore the reason for failing to retreive a clause
impl From<ClauseStoreErr> for AnalysisError {
    fn from(_: ClauseStoreErr) -> Self {
        AnalysisError::ClauseStore
    }
}

pub enum ReportError {
    StoreFailure, // Failure to retreive a clause from the store for any reason
    UnsatCoreUnavailable,
}

// Ignore the reason for failing to retreive a clause
impl From<ClauseStoreErr> for ReportError {
    fn from(_: ClauseStoreErr) -> Self {
        ReportError::StoreFailure
    }
}
