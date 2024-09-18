use crate::structures::LevelIndex;

pub type VariableId = usize;

#[derive(Clone, Debug)]
pub struct Variable {
    pub name: String,
    pub decision_level: Option<LevelIndex>,
    pub id: VariableId,
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
