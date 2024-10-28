use crate::{
    config::ActivityConflict, context::store::ClauseKey, generic::fixed_index::FixedIndex,
    structures::level::LevelIndex,
};
use std::cell::UnsafeCell;
use std::marker::PhantomPinned;

pub mod core;
pub mod delegate;
pub mod list;

pub type VariableId = u32;

pub struct Variable {
    _pin: PhantomPinned,
    name: String,
    id: VariableId,
    polarity: UnsafeCell<Option<bool>>,
    previous_polarity: UnsafeCell<Option<bool>>,
    decision_level: UnsafeCell<Option<LevelIndex>>,
    positive_occurrences: UnsafeCell<Vec<ClauseKey>>,
    negative_occurrences: UnsafeCell<Vec<ClauseKey>>,
    activity: UnsafeCell<ActivityConflict>,
}

#[derive(Debug)]
pub enum Status {
    Set,
    Match,
    Conflict,
}

impl FixedIndex for Variable {
    fn index(&self) -> usize {
        self.id as usize
    }
}
