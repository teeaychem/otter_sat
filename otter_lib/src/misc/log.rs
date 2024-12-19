//! Miscelanous items related to [logging](log).
//!
//! Calls to the log macro are made throughout the library.
//! These are intended to provide useful information for extending the library and/or fixing issues with the library.
//!
//! If you're interested in keeping track of learnt formulas, etc. you may wish to use [dispatches](crate::dispatch) instead.
//!
//! Note, no log implementation is provided.
//! For more details, see [log].

/// Targets to be used within a [log]! macro.
pub mod targets {
    pub const PROPAGATION: &str = "propagation";
    pub const ANALYSIS: &str = "analysis";
    pub const REDUCTION: &str = "reduction";
    pub const CLAUSE_DB: &str = "clause_db";
    pub const VALUATION: &str = "valuation";
    pub const BACKJUMP: &str = "backjump";
    pub const PREPROCESSING: &str = "preprocessing";
    pub const RESOLUTION: &str = "resolution";
    pub const SUBSUMPTION: &str = "subsumption";
    pub const TRANSFER: &str = "transfer";
    pub const QUEUE: &str = "queue";
}
