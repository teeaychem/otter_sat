use std::time::Duration;

/// Counts for various things which count, roughly.
pub struct Counters {
    /// A count of every conflict seen during a solve.
    pub total_conflicts: usize,

    /// A count of conflicts seen since the last restart.
    ///
    /// As u32 rather than a usize for easier interaction with scheduling variables.
    pub fresh_conflicts: u32,

    /// A count of all decisions made.
    pub total_decisions: usize,

    /// The total number of iterations through a solve.
    pub total_iterations: usize,

    /// The number of restarts through a solve.
    pub restarts: usize,

    /// The time taken during a solve.
    pub time: Duration,

    /// The current element in the luby sequence.
    pub luby: crate::generic::luby::Luby,
}

impl Default for Counters {
    fn default() -> Self {
        Counters {
            fresh_conflicts: 0,

            total_decisions: 0,
            total_iterations: 0,
            total_conflicts: 0,

            restarts: 0,
            time: Duration::from_secs(0),

            luby: crate::generic::luby::Luby::default(),
        }
    }
}
