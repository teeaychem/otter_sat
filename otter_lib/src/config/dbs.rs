use super::{Activity, GlueStrength};

#[derive(Clone)]
pub struct ClauseDBConfig {
    pub activity_increment: Activity,
    pub activity_decay: Activity,
    pub max_activity: Activity,
    pub glue_strength: GlueStrength,
}

impl Default for ClauseDBConfig {
    fn default() -> Self {
        ClauseDBConfig {
            activity_increment: 1.0,
            activity_decay: 50.0 * 1e-3,
            max_activity: (2.0 as Activity).powi(512),
            glue_strength: 2,
        }
    }
}

#[derive(Clone)]
pub struct VariableDBConfig {
    /// The amount with which to bump a variable by when applying VSIDS.
    pub bump: Activity,

    /// After a conflict increase the variable bump by a value (proportional to) 1 / (1 - `FACTOR`^-3)
    pub bump_decay: Activity,

    /// The maximum value to which the activity a variable can rise before rescoring the activity of all variables.
    pub bump_max: Activity,
}

impl Default for VariableDBConfig {
    fn default() -> Self {
        VariableDBConfig {
            bump: 1.0,
            bump_decay: 50.0 * 1e-3,
            bump_max: (2.0 as Activity).powi(512), // activity_max: 1e150,
        }
    }
}
