use crate::{
    context::level::{Level, LevelIndex},
    structures::literal::{Literal, Source},
};

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
