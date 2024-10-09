use crate::structures::{level::LevelIndex, solve::ClauseKey};

pub type VariableId = usize;
use std::cell::Cell;

pub struct Variable {
    name: String,
    id: VariableId,
    decision_level: Cell<Option<LevelIndex>>,
    positive_watch_occurrences: Cell<Vec<ClauseKey>>,
    negative_watch_occurrences: Cell<Vec<ClauseKey>>,
    activity: Cell<ActivityRep>,
}

type ActivityRep = f32;

impl Variable {
    pub fn new(name: &str, id: VariableId) -> Self {
        Variable {
            name: name.to_string(),
            decision_level: Cell::new(None),
            id,
            positive_watch_occurrences: Cell::new(Vec::new()),
            negative_watch_occurrences: Cell::new(Vec::new()),
            activity: Cell::new(0.0),
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn decision_level(&self) -> Option<LevelIndex> {
        self.decision_level.get()
    }

    pub fn clear_decision_level(&self) {
        self.decision_level.set(None);
    }

    pub fn set_decision_level(&self, level: LevelIndex) {
        self.decision_level.set(Some(level))
    }

    pub fn id(&self) -> VariableId {
        self.id
    }

    pub fn add_activity(&self, by: ActivityRep) {
        let mut activity = self.activity.get();
        activity += by;
        self.activity.set(activity);
    }

    pub fn multiply_activity(&self, by: ActivityRep) {
        self.activity.set(self.activity.get() * by);
    }

    pub fn activity(&self) -> ActivityRep {
        self.activity.get()
    }

    pub fn watch_removed(&self, clause_key: ClauseKey, polarity: bool) {
        match polarity {
            true => {
                let mut temporary = self.positive_watch_occurrences.take();
                let position = temporary.iter().position(|sc| *sc == clause_key);
                if let Some(p) = position {
                    temporary.swap_remove(p);
                }
                self.positive_watch_occurrences.set(temporary);
            }
            false => {
                let mut temporary = self.negative_watch_occurrences.take();
                let position = temporary.iter().position(|sc| *sc == clause_key);
                if let Some(p) = position {
                    temporary.swap_remove(p);
                }
                self.negative_watch_occurrences.set(temporary);
            }
        };
    }

    pub fn watch_added(&self, clause_key: ClauseKey, polarity: bool) {
        match polarity {
            true => {
                let mut temporary = self.positive_watch_occurrences.take();
                temporary.push(clause_key);
                self.positive_watch_occurrences.set(temporary);
            }
            false => {
                let mut temporary = self.negative_watch_occurrences.take();
                temporary.push(clause_key);
                self.negative_watch_occurrences.set(temporary);
            }
        }
    }

    pub fn take_occurrence_vec(&self, polarity: bool) -> Vec<ClauseKey> {
        match polarity {
            true => self.positive_watch_occurrences.take(),
            false => self.negative_watch_occurrences.take(),
        }
    }

    pub fn restore_occurrence_vec(&self, polarity: bool, vec: Vec<ClauseKey>) {
        match polarity {
            true => self.positive_watch_occurrences.set(vec),
            false => self.negative_watch_occurrences.set(vec),
        };
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
