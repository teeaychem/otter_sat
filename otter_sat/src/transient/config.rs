use crate::{
    config::{Config, StoppingCriteria},
    db::clause::callbacks::CallbackOnResolution,
};

/// Configuration for a resolution buffer.
pub struct BufferConfig {
    /// Whether check for and initiate subsumption.
    pub subsumption: bool,

    /// The stopping criteria to use during resolution.
    pub stopping: StoppingCriteria,

    /// The callback used on completion
    pub callback: Option<Box<CallbackOnResolution>>,
}

impl From<&Config> for BufferConfig {
    fn from(value: &Config) -> Self {
        Self {
            subsumption: value.switch.subsumption,
            stopping: value.stopping_criteria,
            callback: None,
        }
    }
}
