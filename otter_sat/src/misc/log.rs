/*!
Miscelanous items related to [logging](log).

Calls to the log macro are made throughout the library.
These are intended to provide useful information for extending the library and/or fixing issues.

Note, no log implementation is provided.
For more details, see [log].
*/

/// Targets to be used within a [log]! macro.
pub mod targets {
    /// Logs related to [BCP](crate::procedures::bcp)
    pub const PROPAGATION: &str = "propagation";

    /// Logs related to [analysis](crate::procedures::analysis)
    pub const ANALYSIS: &str = "analysis";

    /// Logs related to clause deletion
    pub const REDUCTION: &str = "reduction";

    /// Logs related to the [clause database](crate::db::clause)
    pub const CLAUSE_DB: &str = "clause_db";

    /// Logs related to a valuation
    pub const VALUATION: &str = "valuation";

    /// Logs related to [backjumping](crate::procedures::backjump)
    pub const BACKJUMP: &str = "backjump";

    /// Logs related to preprocessing
    pub const PREPROCESSING: &str = "preprocessing";

    /// Logs related to [resolution](crate::atom_cells)
    pub const ATOMCELLS: &str = "resolution";

    /// Logs related to subsumption
    pub const SUBSUMPTION: &str = "subsumption";
}
