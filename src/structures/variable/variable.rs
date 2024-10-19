use crate::structures::variable::{Variable, VariableId, ActivityRep};
use crate::{context::store::ClauseKey, structures::level::LevelIndex};

use std::cell::UnsafeCell;

impl Variable {
    pub fn new(name: &str, id: VariableId) -> Self {
        Self {
            name: name.to_string(),
            decision_level: UnsafeCell::new(None),
            id,
            polarity: None,
            positive_occurrences: UnsafeCell::new(Vec::with_capacity(512)),
            negative_occurrences: UnsafeCell::new(Vec::with_capacity(512)),
            activity: UnsafeCell::new(0.0),
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn decision_level(&self) -> Option<LevelIndex> {
        unsafe { *self.decision_level.get() }
    }

    pub fn set_decision_level(&self, level: LevelIndex) {
        unsafe { *self.decision_level.get() = Some(level) }
    }

    pub const fn id(&self) -> VariableId {
        self.id
    }

    pub const fn index(&self) -> usize {
        self.id as usize
    }

    pub fn add_activity(&self, by: ActivityRep) {
        unsafe { *self.activity.get() += by }
    }

    pub fn multiply_activity(&self, by: ActivityRep) {
        unsafe { *self.activity.get() = *self.activity.get() * by }
    }

    pub fn activity(&self) -> ActivityRep {
        unsafe { *self.activity.get() }
    }

    pub fn watch_added(&self, clause_key: ClauseKey, polarity: bool) {
        let occurrences = match polarity {
            true => unsafe { &mut *self.positive_occurrences.get() },
            false => unsafe { &mut *self.negative_occurrences.get() },
        };
        occurrences.push(clause_key);
    }

    pub fn polarity(&self) -> Option<bool> {
        self.polarity
    }

    pub fn set_polarity(&mut self, polarity: Option<bool>) {
        self.polarity = polarity
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
