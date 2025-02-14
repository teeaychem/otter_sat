use crate::context::ContextState;

/// Details of a configuration option.
#[derive(Clone)]
pub struct ConfigOption<T> {
    /// The name used to identify the option.
    pub name: &'static str,

    /// The minimum value of the option.
    pub min: T,

    /// The maximum value of the option.
    pub max: T,

    /// The minimum state the option may be set in.
    pub max_state: ContextState,

    /// The value of the option.
    pub value: T,
}

impl<T: Clone> ConfigOption<T> {
    /// Return the limits of the option as a (min, max) pair.
    pub fn min_max(&self) -> (T, T) {
        (self.min.clone(), self.max.clone())
    }
}
