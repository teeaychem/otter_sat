pub mod store;

use crate::{
    context::stores::LevelIndex,
    structures::literal::{Literal, LiteralSource, LiteralTrait},
};

#[derive(Debug)]
pub struct KnowledgeLevel {
    observations: Vec<Literal>,
}

#[derive(Debug)]
pub struct DecisionLevel {
    choice: Literal,
    observations: Vec<(LiteralSource, Literal)>,
}

pub struct LevelStore {
    knowledge: KnowledgeLevel,
    levels: Vec<DecisionLevel>,
}

use std::borrow::Borrow;

impl DecisionLevel {
    pub fn new(literal: Literal) -> Self {
        Self {
            choice: literal,
            observations: vec![],
        }
    }

    pub fn consequences(&self) -> &[(LiteralSource, Literal)] {
        &self.observations
    }
}

impl DecisionLevel {
    fn record_literal<L: Borrow<impl LiteralTrait>>(&mut self, literal: L, source: LiteralSource) {
        self.observations
            .push((source, literal.borrow().canonical()))
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
}
