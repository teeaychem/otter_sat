use super::Clause;

/// A rough distinction between clauses, based on number of literals.
///
/// Care should be taken with the distinction between long clauses and other clauses.
/// For, if subsumption is permitted, a clause with *n* literals may be refined to a clause with two literals.
///
pub enum ClauseKind {
    /// The clause is empty
    Empty,

    /// The clause is a single literal.
    Unit,

    /// The clause is exactly two literals (and was exactly two literals when added to the clause database).
    Binary,

    /// The clause had at least three literals when added to the clause database.
    Long,
}

impl ClauseKind {
    /// Identifies the kind of a clause.
    pub fn identify(clause: &impl Clause) -> Self {
        match clause.size() {
            0 => Self::Empty,

            1 => Self::Unit,

            2 => Self::Binary,

            _long_clause => Self::Long,
        }
    }
}
