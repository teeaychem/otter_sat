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
