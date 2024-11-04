pub mod instance;
pub mod store;

use crate::{
    context::stores::LevelIndex,
    structures::literal::{Literal, LiteralSource},
};

#[derive(Debug)]
pub struct Level {
    index: LevelIndex,
    choice: Option<Literal>,
    observations: Vec<(LiteralSource, Literal)>,
}

pub struct LevelStore {
    levels: Vec<Level>,
}
