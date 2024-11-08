use std::borrow::Borrow;

use crate::{
    context::stores::{level::DecisionLevel, LevelIndex},
    structures::literal::{Literal, LiteralSource, LiteralTrait},
};

use super::KnowledgeLevel;

impl DecisionLevel {
    pub fn new() -> Self {
        Self {
            choice: None,
            observations: vec![],
        }
    }

    pub fn record_literal<L: Borrow<impl LiteralTrait> + Copy>(
        &mut self,
        literal: L,
        source: LiteralSource,
    ) {
        match source {
            LiteralSource::Choice => self.choice = Some(literal.borrow().canonical()),
            LiteralSource::Pure
            | LiteralSource::Assumption
            | LiteralSource::Resolution(_)
            | LiteralSource::Analysis(_)
            | LiteralSource::BCP(_)
            | LiteralSource::Missed(_) => self
                .observations
                .push((source, literal.borrow().canonical())),
        }
    }

    pub fn observations(&self) -> &[(LiteralSource, Literal)] {
        &self.observations
    }

    pub fn literals(&self) -> impl Iterator<Item = Literal> + '_ {
        self.choice.into_iter().chain(
            self.observations
                .iter()
                .map(|(_, literal)| literal)
                .copied(),
        )
    }
}

#[allow(clippy::derivable_impls)]
impl Default for KnowledgeLevel {
    fn default() -> Self {
        Self {
            observations: Vec::default(),
        }
    }
}

impl KnowledgeLevel {
    pub fn record_literal<L: Borrow<impl LiteralTrait>>(&mut self, literal: L) {
        self.observations.push(literal.borrow().canonical())
    }

    pub fn literals(&self) -> &[Literal] {
        &self.observations
    }
}
