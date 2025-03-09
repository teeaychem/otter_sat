use crate::{config::ConfigOption, context::ContextState};

/// Configuration of the literal database.
#[derive(Clone)]
pub struct LiteralDBConfig {
    /// Whether to stack assumptions on individual levels, or combine all assumptions on a single level.
    pub stacked_assumptions: ConfigOption<bool>,
}

impl Default for LiteralDBConfig {
    fn default() -> Self {
        Self {
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
