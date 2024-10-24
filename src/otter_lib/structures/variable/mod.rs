use crate::{context::store::ClauseKey, structures::level::LevelIndex};
use std::cell::UnsafeCell;

pub mod core;
pub mod delegate;
pub mod list;

pub type VariableId = u32;

pub struct Variable {
    name: String,
    id: VariableId,
    polarity: UnsafeCell<Option<bool>>,
    previous_polarity: UnsafeCell<Option<bool>>,
    decision_level: UnsafeCell<Option<LevelIndex>>,
    positive_occurrences: UnsafeCell<Vec<ClauseKey>>,
    negative_occurrences: UnsafeCell<Vec<ClauseKey>>,
    activity: UnsafeCell<ActivityRep>,
}

type ActivityRep = f32;

#[derive(Debug)]
pub enum Status {
    Set,
    Match,
    Conflict,
}
