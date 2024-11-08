use crate::context::stores::{
    level::{DecisionLevel, LevelStore},
    LevelIndex,
};

use super::KnowledgeLevel;

impl Default for LevelStore {
    fn default() -> Self {
        let mut the_store = LevelStore {
            knowledge: KnowledgeLevel::default(),
            levels: Vec::default(),
        };
        the_store.levels.push(DecisionLevel::new());
        the_store
    }
}

impl LevelStore {
    pub fn with_capacity(capacity: usize) -> Self {
        let mut the_store = LevelStore {
            knowledge: KnowledgeLevel::default(),
            levels: Vec::with_capacity(capacity),
        };
        the_store.levels.push(DecisionLevel::new());
        the_store
    }

    pub fn get(&self, index: LevelIndex) -> &DecisionLevel {
        self.levels.get(index).expect("mising level")
    }

    pub fn get_mut(&mut self, index: LevelIndex) -> &mut DecisionLevel {
        self.levels.get_mut(index).expect("mising level")
    }

    pub fn index(&self) -> usize {
        self.levels.len() - 1
    }

    pub fn get_fresh(&mut self) -> LevelIndex {
        let index = self.levels.len();
        self.levels.push(DecisionLevel::new());
        index
    }

    pub fn top(&self) -> &DecisionLevel {
        unsafe { self.levels.get_unchecked(self.index()) }
    }

    pub fn top_mut(&mut self) -> &mut DecisionLevel {
        let index = self.index();
        unsafe { self.levels.get_unchecked_mut(index) }
    }

    pub fn zero(&self) -> &KnowledgeLevel {
        &self.knowledge
    }

    pub fn zero_mut(&mut self) -> &mut KnowledgeLevel {
        &mut self.knowledge
    }

    pub fn pop(&mut self) -> Option<DecisionLevel> {
        self.levels.pop()
    }
}
