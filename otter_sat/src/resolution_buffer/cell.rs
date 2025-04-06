/// Cells of a resolution buffer.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Cell {
    /// Initial valuation
    Value(Option<bool>),

    /// The atom was not valued.
    Clause(bool),

    /// The atom had a conflicting value.
    Conflict(bool),

    /// The atom was part of resolution but was already proven.
    Strengthened,

    /// The atom was used as a pivot when reading a clause into the buffer.
    Pivot,
}
