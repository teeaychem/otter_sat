use crate::{db::keys::ClauseKey, structures::literal::Literal};

// The source of a clause
#[derive(Clone, Copy, Debug)]
pub enum ClauseSource {
    Formula,    // Read from a formula
    Resolution, // Derived via resolution (during analysis, etc.)
}

// The status of a watched literal
#[derive(Clone, Copy, PartialEq)]
pub enum WatchStatus {
    Witness, // The value of the variable of the literal matches the polarity of the literal and so 'witnesses' the truth of the clause
    None,    // The variable of the literal has no value
    Conflict, // The value of the variable of the literal 'conflicts' with the polarity of the literal
}

#[derive(Debug)]
pub enum WatchElement {
    Binary(Literal, ClauseKey),
    Clause(ClauseKey),
}
