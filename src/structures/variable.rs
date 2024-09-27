use crate::structures::{ClauseSource, LevelIndex, StoredClause};

pub type VariableId = usize;
use std::cell::Cell;
use std::collections::BTreeSet;
use std::rc::Rc;

#[derive(Clone, Debug)]
pub struct Variable {
    name: String,
    decision_level: Option<LevelIndex>,
    id: VariableId,
    positive_occurrences: Vec<Rc<StoredClause>>,
    negative_occurrences: Vec<Rc<StoredClause>>,
    watch_occurrences: BTreeSet<Rc<StoredClause>>,
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
            watch_occurrences: BTreeSet::new(),
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

    pub fn note_occurence(
        &mut self,
        stored_clause: Rc<StoredClause>,
        polarity: bool,
    ) {
        match polarity {
            true => self.positive_occurrences.push(stored_clause),
            false => self.negative_occurrences.push(stored_clause),
        }
    }

    pub fn occurrences(&self) -> impl Iterator<Item = Rc<StoredClause>> + '_ {
        self.positive_occurrences
            .iter()
            .chain(&self.negative_occurrences)
            .cloned()
    }

    pub fn watch_occurrences(&self) -> impl Iterator<Item = Rc<StoredClause>> + '_ {
        self.watch_occurrences.iter().cloned()
    }

    pub fn watch_removed(&mut self, stored_clause: &Rc<StoredClause>) {
        self.watch_occurrences.remove(stored_clause);
    }

    pub fn watch_added(&mut self, stored_clause: Rc<StoredClause>) {
        self.watch_occurrences.insert(stored_clause);
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
