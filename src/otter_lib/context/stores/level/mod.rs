pub mod store;

use crossbeam::channel::Sender;

use crate::{
    context::stores::LevelIndex,
    dispatch::Dispatch,
    structures::literal::{Literal, LiteralSource, LiteralTrait},
};

/*
A struct abstracting over decision levels.
Internally this makes use of a pair of private structs.
Though, this should probably be revised at some point…

- KnowledgeLevel
  Aka. decision level zero
  This contains assumptions or proven literals

- DecisionLevel
  A choice and the consequences of that choice

Specifically, each structs can be replaced by a simple vec.
And, for decision levels a stack of pointers to where the level began would work.
The choice/consequence distinction requires some attention, though.

For now, this works ok.
 */

pub struct LevelStore {
    knowledge: KnowledgeLevel,
    levels: Vec<DecisionLevel>,
    tx: Sender<Dispatch>,
}

impl LevelStore {
    pub fn new(tx: Sender<Dispatch>) -> Self {
        LevelStore {
            knowledge: KnowledgeLevel::default(),
            levels: Vec::default(),
            tx,
        }
    }
}

#[derive(Debug)]
struct KnowledgeLevel {
    observations: Vec<Literal>,
}

#[derive(Debug)]
struct DecisionLevel {
    choice: Literal,
    consequences: Vec<(LiteralSource, Literal)>,
}

use std::borrow::Borrow;

impl DecisionLevel {
    pub fn new(literal: Literal) -> Self {
        Self {
            choice: literal,
            consequences: vec![],
        }
    }

    pub fn consequences(&self) -> &[(LiteralSource, Literal)] {
        &self.consequences
    }
}

impl DecisionLevel {
    fn record_consequence<L: Borrow<impl LiteralTrait>>(
        &mut self,
        literal: L,
        source: LiteralSource,
    ) {
        self.consequences
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
