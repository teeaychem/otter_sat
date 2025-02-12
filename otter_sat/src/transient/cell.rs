use crate::structures::literal::CLiteral;

/// Cells of a resolution buffer.
#[derive(Clone, Copy)]
pub enum Cell {
    /// Initial valuation
    Value(Option<bool>),

    /// The atom was not valued.
    None(CLiteral),

    /// The atom had a conflicting value.
    Conflict(CLiteral),

    /// The atom was part of resolution but was already proven.
    Strengthened,

    /// The atom was used as a pivot when reading a clause into the buffer.
    Pivot,
}
