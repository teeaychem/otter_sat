use crate::{
    db::keys::{ChoiceIndex, VariableIndex},
    types::clause::WatchElement,
};
use std::cell::UnsafeCell;

pub mod variable_impl;

pub struct Variable {
    id: VariableIndex,
    value: UnsafeCell<Option<bool>>,
    previous_value: UnsafeCell<bool>,
    choice: UnsafeCell<Option<ChoiceIndex>>,
    positive_occurrences: UnsafeCell<Vec<WatchElement>>,
    positive_occurrences_binary: UnsafeCell<Vec<WatchElement>>,
    negative_occurrences: UnsafeCell<Vec<WatchElement>>,
    negative_occurrences_binary: UnsafeCell<Vec<WatchElement>>,
}
