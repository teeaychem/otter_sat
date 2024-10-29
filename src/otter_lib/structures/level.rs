use std::fmt::Debug;

use crate::structures::literal::{Literal, Source};

pub type LevelIndex = usize;

#[derive(Debug)]
pub struct Level {
    index: LevelIndex,
    choice: Option<Literal>,
    observations: Vec<(Source, Literal)>,
}

impl Level {
    pub fn new(index: LevelIndex) -> Self {
        Self {
            index,
            choice: None,
            observations: vec![],
        }
    }

    pub fn index(&self) -> LevelIndex {
        self.index
    }

    pub fn record_literal(&mut self, literal: Literal, source: Source) {
        match source {
            Source::Choice => self.choice = Some(literal),
            Source::Pure | Source::Assumption | Source::Resolution | Source::Clause(_) => {
                self.observations.push((source, literal))
            }
        }
    }

    pub fn observations(&self) -> &[(Source, Literal)] {
        &self.observations
    }

    pub fn extend_observations(&mut self, with: Vec<(Source, Literal)>) {
        self.observations.extend(with);
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
