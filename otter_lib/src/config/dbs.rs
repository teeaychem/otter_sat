//! Configuration for clause and variable databases.

use super::{Activity, GlueStrength};

#[derive(Clone, Debug)]
pub struct ClauseDBConfig {
    /// The activity with which the next variable bumped will be bumped by, dynamically adjusted.
    pub bump: Activity,

    /// The decay to the activity of a variable each conflict.
    pub decay: Activity,

    /// The maximum activity any variable may have before the activity of all variables is compressed.
    pub max_bump: Activity,

    /// Any clauses with lbd within the lbd bound (lbd ≤ bound) will not be removed from the clause database.
    pub lbd_bound: GlueStrength,
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

#[derive(Clone, Debug)]
pub struct VariableDBConfig {
    /// The amount with which to bump a variable by when applying VSIDS.
    pub bump: Activity,

    /// After a conflict increase the variable bump by a value (proportional to) 1 / (1 - `FACTOR`^-3)
    pub decay: Activity,

    /// The maximum value to which the activity a variable can rise before rescoring the activity of all variables.
    pub max_bump: Activity,
}

impl Default for VariableDBConfig {
    fn default() -> Self {
        VariableDBConfig {
            bump: 1.0,
            decay: 50.0 * 1e-3,
            max_bump: (2.0 as Activity).powi(512), // activity_max: 1e150,
        }
    }
}
