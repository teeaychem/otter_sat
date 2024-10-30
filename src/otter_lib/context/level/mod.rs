pub mod instance;
pub mod store;

use crate::structures::literal::{Literal, Source};

pub type LevelIndex = usize;

#[derive(Debug)]
pub struct Level {
    index: LevelIndex,
    choice: Option<Literal>,
    observations: Vec<(Source, Literal)>,
}

pub struct LevelStore {
    levels: Vec<Level>,
}
