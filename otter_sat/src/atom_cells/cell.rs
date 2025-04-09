use crate::{db::LevelIndex, structures::consequence::AssignmentSource};

#[derive(Clone)]
pub enum ResolutionStatus {
    /// Initial valuation
    Valuation,

    /// Backjumped from
    Backjump,

    /// The atom has been merged into the clause, but had no value.
    Asserted,

    /// The atom has been merged into the clause, and had some conflicting value.
    Asserting,

    /// The atom has been merged into the clause, but has been proven.
    Strengthened,

    /// The atom has been merged into the clause, and was used as a pivot.
    Pivot,

    /// A proven literal
    Proven,
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
    pub status: ResolutionStatus,
    pub level: Option<LevelIndex>,
}

impl Default for AtomCell {
    fn default() -> Self {
        AtomCell {
            value: None,
            source: AssignmentSource::None,
            status: ResolutionStatus::Valuation,
            level: None,
            previous_value: false,
        }
    }
}
