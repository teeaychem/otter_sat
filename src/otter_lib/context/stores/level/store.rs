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
            LiteralSource::Assumption => self.knowledge.record_literal(literal),
            LiteralSource::Pure => self.knowledge.record_literal(literal),
            _ => self.top_mut().record_literal(literal, source),
        }
    }

    pub fn decision_made(&self) -> bool {
        self.levels.len() > 1
    }

    pub fn decision_count(&self) -> usize {
        self.levels.len() - 1
    }
}

impl LevelStore {
    fn index(&self) -> usize {
        self.levels.len() - 1
    }

    fn top_mut(&mut self) -> &mut DecisionLevel {
        let index = self.index();
        unsafe { self.levels.get_unchecked_mut(index) }
    }

    fn get(&self, index: LevelIndex) -> &DecisionLevel {
        if index == 0 {
            panic!("hm")
        };
        self.levels.get(index).expect("mising level")
    }

    fn get_mut(&mut self, index: LevelIndex) -> &mut DecisionLevel {
        if index == 0 {
            panic!("hm mut")
        };
        self.levels.get_mut(index).expect("mising level")
    }
}
