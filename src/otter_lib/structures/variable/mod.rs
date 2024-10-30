use crate::context::{level::LevelIndex, store::ClauseKey};
use std::cell::UnsafeCell;

pub mod core;
pub mod delegate;
pub mod list;

pub type VariableId = u32;

pub struct Variable {
    id: VariableId,
    value: UnsafeCell<Option<bool>>,
    previous_value: UnsafeCell<Option<bool>>,
    decision_level: UnsafeCell<Option<LevelIndex>>,
    positive_occurrences: UnsafeCell<Vec<ClauseKey>>,
    negative_occurrences: UnsafeCell<Vec<ClauseKey>>,
}

#[derive(Debug)]
pub enum Status {
    Set,
    Match,
    Conflict,
}
