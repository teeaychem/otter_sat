use super::Clause;

/// A rough distinction between clauses, based on number of literals.
pub enum ClauseKind {
    /// The clause is empty
    Empty,

    /// The clause is a single literal.
    Unit,

    /// The clause is exactly two literals.
    Binary,

    /// The clause is (inexactly) more than two literals.
    Long,
}

impl ClauseKind {
    /// Identifies what kind of a clause a clause is.
    pub fn identify(clause: &impl Clause) -> Self {
        match clause.size() {
            0 => Self::Empty,
            1 => Self::Unit,
            2 => Self::Binary,
            _ => Self::Long,
        }
    }
}
