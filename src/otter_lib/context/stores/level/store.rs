use crate::{
    context::stores::{
        level::{DecisionLevel, LevelStore},
        LevelIndex,
    },
    structures::literal::{Literal, LiteralSource},
};

use super::KnowledgeLevel;

impl Default for LevelStore {
    fn default() -> Self {
        let mut the_store = LevelStore {
            knowledge: KnowledgeLevel::default(),
            levels: Vec::default(),
        };
        the_store.levels.push(DecisionLevel::new(None));
        the_store
    }
}

impl LevelStore {
    fn get(&self, index: LevelIndex) -> &DecisionLevel {
        self.levels.get(index).expect("mising level")
    }

    fn get_mut(&mut self, index: LevelIndex) -> &mut DecisionLevel {
        self.levels.get_mut(index).expect("mising level")
    }
}

impl LevelStore {
    pub fn index(&self) -> usize {
        self.levels.len() - 1
    }

    pub fn make_choice(&mut self, choice: Literal) {
        let mut level = DecisionLevel::new(Some(choice));
        self.levels.push(level);
    }

    pub fn top(&self) -> &DecisionLevel {
        unsafe { self.levels.get_unchecked(self.index()) }
    }

    pub fn zero(&self) -> &KnowledgeLevel {
        &self.knowledge
    }

    pub fn pop(&mut self) -> Option<DecisionLevel> {
        self.levels.pop()
    }

    pub fn record_literal(&mut self, literal: Literal, source: LiteralSource) {
        match source {
            LiteralSource::Choice => {}
            LiteralSource::Assumption => self.zero_mut().record_literal(literal),
            LiteralSource::Pure => self.zero_mut().record_literal(literal),
            _ => self.top_mut().record_literal(literal, source),
        }
    }
}

impl LevelStore {
    fn top_mut(&mut self) -> &mut DecisionLevel {
        let index = self.index();
        unsafe { self.levels.get_unchecked_mut(index) }
    }

    fn zero_mut(&mut self) -> &mut KnowledgeLevel {
        &mut self.knowledge
    }
}
