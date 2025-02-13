use crate::config::{Config, StoppingCriteria};

/// Configuration for a resolution buffer.
pub struct BufferConfig {
    /// Whether check for and initiate subsumption.
    pub subsumption: bool,

    /// The stopping criteria to use during resolution.
    pub stopping: StoppingCriteria,
}

impl From<&Config> for BufferConfig {
    fn from(value: &Config) -> Self {
        Self {
            subsumption: value.subsumption.value,
            stopping: value.stopping_criteria.value,
        }
    }
}
