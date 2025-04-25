use crate::{db::LevelIndex, structures::consequence::AssignmentSource};

#[derive(Clone, PartialEq, Eq)]
pub enum ResolutionFlag {
    /// Initial valuation.
    Valuation,

    /// Backjumped from.
    Backjump,

    /// The atom has been merged into the clause, but had no value.
    Asserted,

    /// The atom has been merged into the clause, and had some conflicting value.
    Asserting,

    /// The atom has been merged into the clause, and was used as a pivot.
    Pivot,

    /// A proven literal.
    Proven,

    /// Used when checking for derivable literals.
    Independent,

    /// Used when checking for derivable literals.
    Derivable,
}

/**
Cells of a resolution buffer.

Cells are designed to intially store information about an assignment and additional metadata to aid resolution.

*/
#[derive(Clone)]
pub struct AtomCell {
    pub value: Option<bool>,
    pub previous_value: bool,
    pub source: AssignmentSource,
    pub resolution_flag: ResolutionFlag,
    pub level: Option<LevelIndex>,
}

impl Default for AtomCell {
    fn default() -> Self {
        AtomCell {
            value: None,
            source: AssignmentSource::None,
            resolution_flag: ResolutionFlag::Valuation,
            level: None,
            previous_value: false,
        }
    }
}
