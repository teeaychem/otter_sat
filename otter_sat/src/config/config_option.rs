use crate::context::ContextState;

#[derive(Clone)]
pub struct ConfigOption<T> {
    pub name: &'static str,
    pub min: T,
    pub max: T,
    pub max_state: ContextState,
    pub value: T,
}
