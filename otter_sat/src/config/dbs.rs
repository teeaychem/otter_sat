//! Configuration of databases, typically derived from the configuration of a context.

use crate::context::ContextState;

use super::{Activity, ConfigOption, LBD};

/// Configuration for the clause database.
#[derive(Clone)]
pub struct ClauseDBConfig {
    /// The activity with which the next atom bumped will be bumped by, dynamically adjusted.
    pub bump: ConfigOption<Activity>,

    /// The decay to the activity of a atom each conflict.
    pub decay: ConfigOption<Activity>,

    /// Any clauses with lbd within the lbd bound (lbd ≤ bound) will not be removed from the clause database.
    pub lbd_bound: ConfigOption<LBD>,
}

impl Default for ClauseDBConfig {
    fn default() -> Self {
        ClauseDBConfig {
            bump: ConfigOption {
                name: "clause_bump",
                min: Activity::MIN,
                max: (2.0 as Activity).powi(512),
                max_state: ContextState::Configuration,
                value: 1.0,
            },

            decay: ConfigOption {
                name: "clause_decay",
                min: Activity::MIN,
                max: Activity::MAX,
                max_state: ContextState::Configuration,
                value: 50.0 * 1e-3,
            },

            lbd_bound: ConfigOption {
                name: "lbd_bound",
                min: LBD::MIN,
                max: LBD::MAX,
                max_state: ContextState::Configuration,
                value: 2,
            },
        }
    }
}

/// Configuration for the atom database.
#[derive(Clone)]
pub struct AtomDBConfig {
    /// The amount with which to bump a atom by when applying [VSIDS](crate::config::vsids).
    pub bump: ConfigOption<Activity>,

    /// After a conflict increase the atom bump by a value (proportional to) 1 / (1 - `FACTOR`^-3)
    pub decay: ConfigOption<Activity>,

    /// Whether to stack assumptions on individual levels, or combine all assumptions on a single level.
    pub stacked_assumptions: ConfigOption<bool>,
}

impl Default for AtomDBConfig {
    fn default() -> Self {
        AtomDBConfig {
            bump: ConfigOption {
                name: "atom_bump",
                min: Activity::MIN,
                max: (2.0 as Activity).powi(512),
                max_state: ContextState::Configuration,
                value: 1.0,
            },

            decay: ConfigOption {
                name: "atom_decay",
                min: Activity::MIN,
                max: Activity::MAX,
                max_state: ContextState::Configuration,
                value: 50.0 * 1e-3,
            },

            stacked_assumptions: ConfigOption {
                name: "stacked_assumptions",
                min: false,
                max: true,
                max_state: ContextState::Configuration,
                value: true,
            },
        }
    }
}
