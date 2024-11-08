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
    choice: Option<Literal>,
    observations: Vec<(LiteralSource, Literal)>,
}

pub struct LevelStore {
    knowledge: KnowledgeLevel,
    levels: Vec<DecisionLevel>,
}

use std::borrow::Borrow;

impl DecisionLevel {
    pub fn new(literal: Option<Literal>) -> Self {
        Self {
            choice: literal,
            observations: vec![],
        }
    }

    fn record_literal<L: Borrow<impl LiteralTrait> + Copy>(
        &mut self,
        literal: L,
        source: LiteralSource,
    ) {
        self.observations
            .push((source, literal.borrow().canonical()))
    }

    pub fn observations(&self) -> &[(LiteralSource, Literal)] {
        &self.observations
    }

    pub fn choice(&self) -> Literal {
        self.choice.unwrap()
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
