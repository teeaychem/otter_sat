use crate::structures::consequence::{Assignment, AssignmentSource};

#[derive(Clone)]
pub enum CellStatus {
    /// Initial valuation
    Valuation,

    /// The atom has been merged into the clause, but had no value.
    Clause,

    /// The atom has been merged into the clause, and had some conflicting value.
    Conflict,

    /// The atom has been merged into the clause, but has been proven.
    Strengthened,

    /// The atom has been merged into the clause, and was used as a pivot.
    Pivot,
}

/**
Cells of a resolution buffer.

Cells are designed to intially store information about an assignment and additional metadata to aid resolution.

*/
#[derive(Clone)]
pub struct Cell {
    pub value: Option<bool>,
    pub source: Option<AssignmentSource>,
    pub status: CellStatus,
}

impl Default for Cell {
    fn default() -> Self {
        Cell {
            value: None,
            source: None,
            status: CellStatus::Valuation,
        }
    }
}

impl From<Assignment> for Cell {
    fn from(assignment: Assignment) -> Self {
        Cell {
            value: Some(assignment.value()),
            source: Some(*assignment.source()),
            status: CellStatus::Valuation,
        }
    }
}
