use crate::structures::{clause::stored_clause::StoredClause, level::LevelIndex};

pub type VariableId = usize;
use std::cell::Cell;
use std::rc::Rc;
pub struct Variable {
    name: String,
    decision_level: Option<LevelIndex>,
    id: VariableId,
    positive_occurrences: Cell<Vec<Rc<StoredClause>>>,
    negative_occurrences: Cell<Vec<Rc<StoredClause>>>,
    pub positive_watch_occurrences: Cell<Vec<Rc<StoredClause>>>,
    pub negative_watch_occurrences: Cell<Vec<Rc<StoredClause>>>,
    activity: Cell<f32>,
}

impl Variable {
    pub fn new(name: &str, id: VariableId) -> Self {
        Variable {
            name: name.to_string(),
            decision_level: None,
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
        self.decision_level
    }

    pub fn clear_decision_level(&mut self) {
        self.decision_level = None
    }

    pub fn set_decision_level(&mut self, level: LevelIndex) {
        self.decision_level = Some(level)
    }

    pub fn id(&self) -> VariableId {
        self.id
    }

    pub fn add_activity(&self, by: f32) {
        let mut activity = self.activity.get();
        activity += by;
        self.activity.replace(activity);
    }

    pub fn divide_activity(&self, by: f32) {
        let mut activity = self.activity.get();
        activity /= by;
        self.activity.replace(activity);
    }

    pub fn activity(&self) -> f32 {
        self.activity.get()
    }

    pub fn note_occurence(&self, stored_clause: &Rc<StoredClause>, polarity: bool) {
        let cloned = stored_clause.clone();
        match polarity {
            true => {
                let mut temporary = self.positive_occurrences.take();
                temporary.push(cloned);
                let _ = self.positive_occurrences.replace(temporary);
            }
            false => {
                let mut temporary = self.negative_occurrences.take();
                temporary.push(cloned);
                let _ = self.negative_occurrences.replace(temporary);
            }
        }
    }

    pub fn note_drop(&self, stored_clause: &Rc<StoredClause>, polarity: bool) {
        match polarity {
            true => {
                let mut temporary = self.positive_occurrences.take();
                let position = temporary.iter().position(|sc| sc == stored_clause);
                if let Some(p) = position {
                    temporary.swap_remove(p);
                }
                let _ = self.positive_occurrences.replace(temporary);
            }
            false => {
                let mut temporary = self.negative_occurrences.take();
                let position = temporary.iter().position(|sc| sc == stored_clause);
                if let Some(p) = position {
                    temporary.swap_remove(p);
                }
                let _ = self.negative_occurrences.replace(temporary);
            }
        }
    }

    pub fn watch_removed(&self, stored_clause: &Rc<StoredClause>, polarity: bool) {
        match polarity {
            true => {
                let mut temporary = self.positive_watch_occurrences.take();
                let position = temporary.iter().position(|sc| sc == stored_clause);
                if let Some(p) = position {
                    temporary.swap_remove(p);
                }
                let _ = self.positive_watch_occurrences.replace(temporary);
            }
            false => {
                let mut temporary = self.negative_watch_occurrences.take();
                let position = temporary.iter().position(|sc| sc == stored_clause);
                if let Some(p) = position {
                    temporary.swap_remove(p);
                }
                let _ = self.negative_watch_occurrences.replace(temporary);
            }
        }
    }

    pub fn watch_added(&self, stored_clause: &Rc<StoredClause>, polarity: bool) {
        match polarity {
            true => {
                let mut temporary = self.positive_watch_occurrences.take();
                temporary.push(stored_clause.clone());
                let _ = self.positive_watch_occurrences.replace(temporary);
            }
            false => {
                let mut temporary = self.negative_watch_occurrences.take();
                temporary.push(stored_clause.clone());
                let _ = self.negative_watch_occurrences.replace(temporary);
            }
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
