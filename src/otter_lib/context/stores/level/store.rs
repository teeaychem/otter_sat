use crate::{
    context::stores::{
        level::{DecisionLevel, LevelStore},
        LevelIndex,
    },
    structures::literal::{Literal, LiteralSource},
};

use super::KnowledgeLevel;

#[allow(clippy::derivable_impls)]
impl Default for LevelStore {
    fn default() -> Self {
        // the_store.levels.push(DecisionLevel::new(None));
        LevelStore {
            knowledge: KnowledgeLevel::default(),
            levels: Vec::default(),
        }
    }
}

impl LevelStore {
    pub fn make_choice(&mut self, choice: Literal) {
        let mut level = DecisionLevel::new(choice);
        self.levels.push(level);
    }

    pub fn top(&self) -> &DecisionLevel {
        unsafe { self.levels.get_unchecked(self.levels.len() - 1) }
    }

    pub fn current_choice(&self) -> Literal {
        unsafe { self.levels.get_unchecked(self.levels.len() - 1).choice }
    }

    pub fn current_consequences(&self) -> &[(LiteralSource, Literal)] {
        unsafe {
            &self
                .levels
                .get_unchecked(self.levels.len() - 1)
                .observations
        }
    }

    pub fn forget_choice(&mut self) -> Option<DecisionLevel> {
        self.levels.pop()
    }

    pub fn proven_literals(&self) -> &[Literal] {
        &self.knowledge.observations
    }

    /*
    Can't assume a decision has been made…

    Choices are ignored, assumptions and pure are always known
    Everything else depends on whether a decision has been made
         */
    pub fn record_literal(&mut self, literal: Literal, source: LiteralSource) {
        // println!("RECORDING {source:?}");
        match source {
            LiteralSource::Choice => {}
            LiteralSource::Assumption => self.knowledge.record_literal(literal),
            LiteralSource::Pure => self.knowledge.record_literal(literal),
            LiteralSource::BCP(_) => match self.levels.len() {
                0 => self.knowledge.record_literal(literal),
                _ => self.top_mut().record_literal(literal, source),
            },
            LiteralSource::Resolution(_) => match self.levels.len() {
                0 => self.knowledge.record_literal(literal),
                _ => self.top_mut().record_literal(literal, source),
            },
            LiteralSource::Analysis(_) => match self.levels.len() {
                0 => self.knowledge.record_literal(literal),
                _ => self.top_mut().record_literal(literal, source),
            },
            LiteralSource::Missed(_) => match self.levels.len() {
                0 => self.knowledge.record_literal(literal),
                _ => self.top_mut().record_literal(literal, source),
            },
            _ => {
                println!("td {source:?}");

                todo!()
            } //self.top_mut().record_literal(literal, source),
        }
    }

    pub fn decision_made(&self) -> bool {
        !self.levels.is_empty()
    }

    pub fn decision_count(&self) -> usize {
        self.levels.len()
    }
}

impl LevelStore {
    fn top_mut(&mut self) -> &mut DecisionLevel {
        let x = self.levels.len() - 1;
        unsafe { self.levels.get_unchecked_mut(x) }
    }

    fn get(&self, index: LevelIndex) -> &DecisionLevel {
        self.levels.get(index).expect("mising level")
    }

    fn get_mut(&mut self, index: LevelIndex) -> &mut DecisionLevel {
        self.levels.get_mut(index).expect("mising level")
    }
}
