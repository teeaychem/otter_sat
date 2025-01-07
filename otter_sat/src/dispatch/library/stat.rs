//! Statistics regarding various things.
use std::time::Duration;

/// Dispatches containing statistics.
#[derive(Clone)]
pub enum Stat {
    /// The count of iterations made.
    Iterations(usize),

    /// The count of decisions made.
    Chosen(usize),

    /// The count of conflicts seen.
    Conflicts(usize),

    /// The time elapsed.
    Time(Duration),
}
