use crate::{db::LevelIndex, structures::consequence::Assignment};

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
    pub assignment: Option<Assignment>,
    pub status: CellStatus,
    pub level: Option<LevelIndex>,
    pub previous_value: bool,
}

impl Cell {
    pub fn value(&self) -> Option<bool> {
        self.assignment.as_ref().map(|a| a.value())
    }

    pub fn get_assignment(&self) -> &Option<Assignment> {
        &self.assignment
    }
}

impl Default for Cell {
    fn default() -> Self {
        Cell {
            value: None,
            assignment: None,
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
