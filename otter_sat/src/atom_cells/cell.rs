use crate::{db::LevelIndex, structures::consequence::AssignmentSource};

#[derive(Clone)]
pub enum CellStatus {
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
    pub level: Option<LevelIndex>,
    pub previous_value: bool,
}

impl Cell {
    pub fn get_assignment_source(&self) -> Option<&AssignmentSource> {
        match &self.source {
            None => None,
            Some(a) => Some(&a),
        }
    }
}

impl Default for Cell {
    fn default() -> Self {
        Cell {
            value: None,
            source: None,
            status: CellStatus::Valuation,
            level: None,
            previous_value: false,
        }
    }
}

// impl From<Assignment> for Cell {
//     fn from(assignment: Assignment) -> Self {
//         Cell {
//             value: Some(assignment.value()),
//             assignment: Some(assignment.clone()),
//             status: CellStatus::Valuation,
//             level: None
//         }
//     }
// }
