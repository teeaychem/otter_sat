use crate::structures::variable::{Variable, VariableId};
use crate::{context::store::ClauseKey, structures::level::LevelIndex};

use std::cell::UnsafeCell;

impl Variable {
    pub fn new(name: &str, id: VariableId) -> Self {
        Self {
            name: name.to_string(),
            decision_level: UnsafeCell::new(None),
            id,
            polarity: UnsafeCell::new(None),
            previous_polarity: UnsafeCell::new(None),
            positive_occurrences: UnsafeCell::new(Vec::with_capacity(512)),
            negative_occurrences: UnsafeCell::new(Vec::with_capacity(512)),
        }
    }

    pub fn name(&self) -> &str {
        &self.name
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

    pub fn polarity(&self) -> Option<bool> {
        unsafe { *self.polarity.get() }
    }

    pub fn previous_polarity(&self) -> Option<bool> {
        unsafe { *self.previous_polarity.get() }
    }

    pub fn set_polarity(&self, polarity: Option<bool>, level: Option<LevelIndex>) {
        unsafe {
            *self.previous_polarity.get() = *self.polarity.get();
            *self.polarity.get() = polarity;
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

impl PartialOrd for Variable {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Variable {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.name.cmp(&other.name)
    }
}

impl PartialEq for Variable {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for Variable {}
