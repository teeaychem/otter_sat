//! Configuration for clause and atom databases.

use super::{Activity, LBD};

/// Configuration for the clause database.
#[derive(Clone, Debug)]
pub struct ClauseDBConfig {
    /// The activity with which the next atom bumped will be bumped by, dynamically adjusted.
    pub bump: Activity,

    /// The decay to the activity of a atom each conflict.
    pub decay: Activity,

    /// The maximum activity any atom may have before the activity of all atoms is compressed.
    pub max_bump: Activity,

    /// Any clauses with lbd within the lbd bound (lbd ≤ bound) will not be removed from the clause database.
    pub lbd_bound: LBD,
}

impl Default for ClauseDBConfig {
    fn default() -> Self {
        ClauseDBConfig {
            bump: 1.0,
            decay: 50.0 * 1e-3,
            max_bump: (2.0 as Activity).powi(512),
            lbd_bound: 2,
        }
    }
}

/// Configuration for the atom database.
#[derive(Clone, Debug)]
pub struct AtomDBConfig {
    /// The amount with which to bump a atom by when applying [VSIDS](crate::config::vsids).
    pub bump: Activity,

    /// After a conflict increase the atom bump by a value (proportional to) 1 / (1 - `FACTOR`^-3)
    pub decay: Activity,

    /// The maximum value to which the activity a atom can rise before rescoring the activity of all atoms.
    pub max_bump: Activity,
}

impl Default for AtomDBConfig {
    fn default() -> Self {
        AtomDBConfig {
            bump: 1.0,
            decay: 50.0 * 1e-3,
            max_bump: (2.0 as Activity).powi(512), // activity_max: 1e150,
        }
    }
}
