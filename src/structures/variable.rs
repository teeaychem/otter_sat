use crate::structures::{level::LevelIndex, solve::ClauseKey};

pub type VariableId = u32;
use std::cell::UnsafeCell;

pub struct Variable {
    name: String,
    id: VariableId,
    decision_level: UnsafeCell<Option<LevelIndex>>,
    pub positive_watch_occurrences: UnsafeCell<Vec<ClauseKey>>,
    pub negative_watch_occurrences: UnsafeCell<Vec<ClauseKey>>,
    activity: UnsafeCell<ActivityRep>,
}

type ActivityRep = f32;

impl Variable {
    pub fn new(name: &str, id: VariableId) -> Self {
        Variable {
            name: name.to_string(),
            decision_level: UnsafeCell::new(None),
            id,
            positive_watch_occurrences: UnsafeCell::new(Vec::with_capacity(512)),
            negative_watch_occurrences: UnsafeCell::new(Vec::with_capacity(512)),
            activity: UnsafeCell::new(0.0),
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn decision_level(&self) -> Option<LevelIndex> {
        unsafe { *self.decision_level.get() }
    }

    pub fn clear_decision_level(&self) {
        unsafe { *self.decision_level.get() = None }
    }

    pub fn set_decision_level(&self, level: LevelIndex) {
        unsafe { *self.decision_level.get() = Some(level) }
    }

    pub fn id(&self) -> VariableId {
        self.id
    }

    pub fn add_activity(&self, by: ActivityRep) {
        unsafe {
            let activity = self.activity.get();
            *activity += by;
        }
    }

    pub fn multiply_activity(&self, by: ActivityRep) {
        unsafe {
            let was = *self.activity.get();
            *self.activity.get() = was * by;
        }
    }

    pub fn activity(&self) -> ActivityRep {
        unsafe { *self.activity.get() }
    }

    pub fn watch_added(&self, clause_key: ClauseKey, polarity: bool) {
        match polarity {
            true => unsafe {
                let occurrences = &mut *self.positive_watch_occurrences.get();
                occurrences.push(clause_key)
            },
            false => unsafe {
                let occurrences = &mut *self.negative_watch_occurrences.get();
                occurrences.push(clause_key);
            },
        }
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
