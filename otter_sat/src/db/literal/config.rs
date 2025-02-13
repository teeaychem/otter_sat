use crate::{config::ConfigOption, context::ContextState};

#[derive(Clone)]
pub struct LiteralDBConfig {
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
                value: false,
            },
        }
    }
}
