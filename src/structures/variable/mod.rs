use crate::{context::store::ClauseKey, structures::level::LevelIndex};
use std::cell::UnsafeCell;

pub mod list;
pub mod store;
pub mod variable;

pub type VariableId = u32;

pub struct Variable {
    name: String,
    id: VariableId,
    polarity: Option<bool>,
    decision_level: UnsafeCell<Option<LevelIndex>>,
    pub positive_occurrences: UnsafeCell<Vec<ClauseKey>>,
    pub negative_occurrences: UnsafeCell<Vec<ClauseKey>>,
    activity: UnsafeCell<ActivityRep>,
}

type ActivityRep = f32;

pub enum Status {
    NotSet,
    Match,
    Conflict,
}
