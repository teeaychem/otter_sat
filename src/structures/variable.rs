use crate::structures::{
    clause::stored_clause::StoredClause, level::LevelIndex, solve::clause_store::ClauseKey,
};

pub type VariableId = usize;
use std::cell::Cell;

pub struct Variable {
    name: String,
    id: VariableId,
    decision_level: Cell<Option<LevelIndex>>,
    positive_occurrences: Cell<Vec<ClauseKey>>,
    negative_occurrences: Cell<Vec<ClauseKey>>,
    positive_watch_occurrences: Cell<Vec<ClauseKey>>,
    negative_watch_occurrences: Cell<Vec<ClauseKey>>,
    activity: Cell<f32>,
}

impl Variable {
    pub fn new(name: &str, id: VariableId) -> Self {
        Variable {
            name: name.to_string(),
            decision_level: Cell::new(None),
            id,
            positive_occurrences: Cell::new(Vec::new()),
            negative_occurrences: Cell::new(Vec::new()),
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

    pub fn add_activity(&self, by: f32) {
        let mut activity = self.activity.get();
        activity += by;
        self.activity.set(activity);
    }

    pub fn divide_activity(&self, by: f32) {
        let mut activity = self.activity.get();
        activity /= by;
        self.activity.set(activity);
    }

    pub fn activity(&self) -> f32 {
        self.activity.get()
    }

    pub fn note_occurence(&self, clause_key: ClauseKey, polarity: bool) {
        match polarity {
            true => {
                let mut temporary = self.positive_occurrences.take();
                temporary.push(clause_key);
                self.positive_occurrences.set(temporary);
            }
            false => {
                let mut temporary = self.negative_occurrences.take();
                temporary.push(clause_key);
                self.negative_occurrences.set(temporary);
            }
        }
    }

    pub fn note_clause_drop(&self, clause_key: ClauseKey, polarity: bool) {
        let mut temporary = match polarity {
            true => self.positive_occurrences.take(),
            false => self.negative_occurrences.take(),
        };

        let position = temporary.iter().position(|sc| *sc == clause_key);
        if let Some(p) = position {
            temporary.swap_remove(p);
        }

        match polarity {
            true => self.positive_occurrences.set(temporary),
            false => self.negative_occurrences.set(temporary),
        };
    }

    pub fn watch_removed(&self, stored_clause: &StoredClause, polarity: bool) {
        let mut temporary = match polarity {
            true => self.positive_watch_occurrences.take(),
            false => self.negative_watch_occurrences.take(),
        };

        let position = temporary.iter().position(|sc| *sc == stored_clause.key);
        if let Some(p) = position {
            temporary.swap_remove(p);
        }

        match polarity {
            true => self.positive_watch_occurrences.set(temporary),
            false => self.negative_watch_occurrences.set(temporary),
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
