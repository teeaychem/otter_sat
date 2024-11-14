pub mod store;

use crossbeam::channel::Sender;

use crate::{dispatch::Dispatch, structures::literal::Literal, types::gen};

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

pub struct LiteralDB {
    proven: ProvenLiterals,
    chosen: Vec<ChosenLiteral>,
    tx: Sender<Dispatch>,
}

impl LiteralDB {
    pub fn new(tx: Sender<Dispatch>) -> Self {
        LiteralDB {
            proven: ProvenLiterals::default(),
            chosen: Vec::default(),
            tx,
        }
    }
}

#[derive(Debug)]
struct ProvenLiterals {
    observations: Vec<Literal>,
}

#[derive(Debug)]
struct ChosenLiteral {
    choice: Literal,
    consequences: Vec<(gen::LiteralSource, Literal)>,
}

use std::borrow::Borrow;

impl ChosenLiteral {
    pub fn new(literal: Literal) -> Self {
        Self {
            choice: literal,
            consequences: vec![],
        }
    }

    #[allow(dead_code)]
    pub fn consequences(&self) -> &[(gen::LiteralSource, Literal)] {
        &self.consequences
    }
}

impl ChosenLiteral {
    fn record_consequence<L: Borrow<Literal>>(&mut self, literal: L, source: gen::LiteralSource) {
        self.consequences.push((source, *literal.borrow()))
    }
}

#[allow(clippy::derivable_impls)]
impl Default for ProvenLiterals {
    fn default() -> Self {
        Self {
            observations: Vec::default(),
        }
    }
}

impl ProvenLiterals {
    pub fn record_literal<L: Borrow<Literal>>(&mut self, literal: L) {
        self.observations.push(*literal.borrow())
    }
}
