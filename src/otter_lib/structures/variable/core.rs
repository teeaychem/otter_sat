use crate::context::{level::LevelIndex, store::ClauseKey};
use crate::structures::variable::{Variable, VariableId};

use std::cell::UnsafeCell;

impl Variable {
    pub fn new(id: VariableId) -> Self {
        Self {
            decision_level: UnsafeCell::new(None),
            id,
            value: UnsafeCell::new(None),
            previous_value: UnsafeCell::new(None),
            positive_occurrences: UnsafeCell::new(Vec::with_capacity(512)),
            negative_occurrences: UnsafeCell::new(Vec::with_capacity(512)),
        }
    }

    pub fn index(&self) -> usize {
        self.id as usize
    }

    pub fn decision_level(&self) -> Option<LevelIndex> {
        unsafe { *self.decision_level.get() }
    }

    pub const fn id(&self) -> VariableId {
        self.id
    }

    pub fn watch_added(&self, clause_key: ClauseKey, polarity: bool) {
        match polarity {
            true => unsafe { &mut *self.positive_occurrences.get() },
            false => unsafe { &mut *self.negative_occurrences.get() },
        }
        .push(clause_key);
    }

    pub fn value(&self) -> Option<bool> {
        unsafe { *self.value.get() }
    }

    pub fn previous_value(&self) -> Option<bool> {
        unsafe { *self.previous_value.get() }
    }

    pub fn set_value(&self, polarity: Option<bool>, level: Option<LevelIndex>) {
        unsafe {
            *self.previous_value.get() = *self.value.get();
            *self.value.get() = polarity;
            *self.decision_level.get() = level
        }
    }

    pub fn occurrence_length(&self, polarity: bool) -> usize {
        match polarity {
            true => unsafe { &*self.positive_occurrences.get() },
            false => unsafe { &*self.negative_occurrences.get() },
        }
        .len()
    }

    pub fn occurrence_key_at_index(&self, polarity: bool, index: usize) -> ClauseKey {
        *unsafe {
            match polarity {
                true => &*self.positive_occurrences.get(),
                false => &*self.negative_occurrences.get(),
            }
            .get_unchecked(index)
        }
    }

    pub fn remove_occurrence_at_index(&self, polarity: bool, index: usize) {
        match polarity {
            true => unsafe { &mut *self.positive_occurrences.get() },
            false => unsafe { &mut *self.negative_occurrences.get() },
        }
        .swap_remove(index);
    }
}
