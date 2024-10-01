use crate::structures::{LevelIndex, StoredClause};

pub type VariableId = usize;
use std::cell::Cell;
use std::rc::Rc;

#[derive(Clone, Debug)]
pub struct Variable {
    name: String,
    decision_level: Option<LevelIndex>,
    id: VariableId,
    positive_occurrences: Vec<Rc<StoredClause>>,
    negative_occurrences: Vec<Rc<StoredClause>>,
    pub positive_watch_occurrences: Vec<Rc<StoredClause>>,
    pub negative_watch_occurrences: Vec<Rc<StoredClause>>,
    activity: Cell<f32>,
}

impl Variable {
    pub fn new(name: &str, id: VariableId) -> Self {
        Variable {
            name: name.to_string(),
            decision_level: None,
            id,
            positive_occurrences: Vec::new(),
            negative_occurrences: Vec::new(),
            positive_watch_occurrences: Vec::new(),
            negative_watch_occurrences: Vec::new(),
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

    pub fn increase_activity(&self, by: f32) {
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

    pub fn note_occurence(&mut self, stored_clause: &Rc<StoredClause>, polarity: bool) {
        let cloned = stored_clause.clone();
        match polarity {
            true => self.positive_occurrences.push(cloned),
            false => self.negative_occurrences.push(cloned),
        }
    }

    pub fn note_drop(&mut self, stored_clause: &Rc<StoredClause>, polarity: bool) {
        match polarity {
            true => {
                if let Some(p) = self
                    .positive_occurrences
                    .iter()
                    .position(|sc| sc == stored_clause)
                {
                    let _ = self.positive_occurrences.swap_remove(p);
                }
            }
            false => {
                if let Some(p) = self
                    .negative_occurrences
                    .iter()
                    .position(|sc| sc == stored_clause)
                {
                    let _ = self.negative_occurrences.swap_remove(p);
                }
            }
        }
    }

    pub fn positive_occurrences(&self) -> &[Rc<StoredClause>] {
        &self.positive_occurrences
    }

    pub fn negative_occurrences(&self) -> &[Rc<StoredClause>] {
        &self.negative_occurrences
    }

    pub fn watch_removed(&mut self, stored_clause: &Rc<StoredClause>, polarity: bool) {
        match polarity {
            true => {
                if let Some(p) = self
                    .positive_watch_occurrences
                    .iter()
                    .position(|sc| sc == stored_clause)
                {
                    self.positive_watch_occurrences.swap_remove(p);
                }
            }
            false => {
                if let Some(p) = self
                    .negative_watch_occurrences
                    .iter()
                    .position(|sc| sc == stored_clause)
                {
                    self.negative_watch_occurrences.swap_remove(p);
                }
            }
        }
    }

    pub fn watch_added(&mut self, stored_clause: &Rc<StoredClause>, polarity: bool) {
        match polarity {
            true => {
                self.positive_watch_occurrences.push(stored_clause.clone());
            }
            false => {
                self.negative_watch_occurrences.push(stored_clause.clone());
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
