use crate::structures::{LevelIndex, ClauseId};

pub type VariableId = usize;

#[derive(Clone, Debug)]
pub struct Variable {
    name: String,
    decision_level: Option<LevelIndex>,
    id: VariableId,
    positive_occurrences: Vec<ClauseId>,
    negative_occurrences: Vec<ClauseId>,
}

impl Variable {
    pub fn new(name: &str, id: VariableId) -> Self {
        Variable {
            name: name.to_string(),
            decision_level: None,
            id,
            positive_occurrences: Vec::new(),
            negative_occurrences: Vec::new(),
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

    pub fn note_occurence(&mut self, clause_id: ClauseId, polarity: bool) {
        match polarity {
            true => self.positive_occurrences.push(clause_id),
            false => self.negative_occurrences.push(clause_id)
        }
    }

    pub fn occurrences(&self) -> impl Iterator<Item = ClauseId> + '_ {
        self.positive_occurrences.iter().chain(&self.negative_occurrences).cloned()
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
