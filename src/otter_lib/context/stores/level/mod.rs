pub mod instance;
pub mod store;

use crate::{
    context::stores::LevelIndex,
    structures::literal::{Literal, LiteralSource},
};

#[derive(Debug)]
pub struct KnowledgeLevel {
    observations: Vec<Literal>,
}

#[derive(Debug)]
pub struct DecisionLevel {
    index: LevelIndex,
    choice: Option<Literal>,
    observations: Vec<(LiteralSource, Literal)>,
}

pub struct LevelStore {
    knowledge: KnowledgeLevel,
    levels: Vec<DecisionLevel>,
}
