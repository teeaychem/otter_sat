use crate::{
    context::stores::{
        level::{DecisionLevel, KnowledgeLevel, LevelStore},
        LevelIndex,
    },
    structures::literal::{Literal, LiteralSource},
};

impl LevelStore {
    pub fn make_choice(&mut self, choice: Literal) {
        self.levels.push(DecisionLevel::new(choice));
    }

    /*
    Can't assume a decision has been made…

    Choices are ignored, assumptions and pure are always known
    Everything else depends on whether a decision has been made
         */
    pub(crate) fn record_literal(&mut self, literal: Literal, source: LiteralSource) {
        match source {
            LiteralSource::Choice => {}
            LiteralSource::Assumption => self.knowledge.record_literal(literal),
            LiteralSource::Pure => self.knowledge.record_literal(literal),
            LiteralSource::BCP(_) => match self.levels.len() {
                0 => self.knowledge.record_literal(literal),
                _ => self.top_mut().record_consequence(literal, source),
            },
            LiteralSource::Resolution(_) => match self.levels.len() {
                0 => self.knowledge.record_literal(literal),
                _ => self.top_mut().record_consequence(literal, source),
            },
            LiteralSource::Analysis(_) => match self.levels.len() {
                0 => self.knowledge.record_literal(literal),
                _ => self.top_mut().record_consequence(literal, source),
            },
            LiteralSource::Missed(_) => match self.levels.len() {
                0 => self.knowledge.record_literal(literal),
                _ => self.top_mut().record_consequence(literal, source),
            },
        }
    }

    pub fn current_choice(&self) -> Literal {
        unsafe { self.levels.get_unchecked(self.levels.len() - 1).choice }
    }

    pub fn current_consequences(&self) -> &[(LiteralSource, Literal)] {
        unsafe {
            &self
                .levels
                .get_unchecked(self.levels.len() - 1)
                .consequences
        }
    }

    pub fn forget_current_choice(&mut self) {
        self.levels.pop();
    }

    pub fn proven_literals(&self) -> &[Literal] {
        &self.knowledge.observations
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
        let level_count = self.levels.len() - 1;
        unsafe { self.levels.get_unchecked_mut(level_count) }
    }
}
