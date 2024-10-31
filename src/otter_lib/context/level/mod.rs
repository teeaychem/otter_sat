pub mod instance;
pub mod store;

use crate::structures::literal::{Literal, LiteralSource};

pub type LevelIndex = usize;

#[derive(Debug)]
pub struct Level {
    index: LevelIndex,
    choice: Option<Literal>,
    observations: Vec<(LiteralSource, Literal)>,
}

pub struct LevelStore {
    levels: Vec<Level>,
}
