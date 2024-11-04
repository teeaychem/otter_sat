use crate::{
    context::stores::{level::Level, LevelIndex},
    structures::literal::{Literal, LiteralSource},
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

    pub fn record_literal(&mut self, literal: Literal, source: LiteralSource) {
        match source {
            LiteralSource::Choice => self.choice = Some(literal),
            LiteralSource::Pure
            | LiteralSource::Assumption
            | LiteralSource::Resolution(_)
            | LiteralSource::Analysis(_)
            | LiteralSource::BCP(_)
            | LiteralSource::Missed(_) => self.observations.push((source, literal)),
        }
    }

    pub fn observations(&self) -> &[(LiteralSource, Literal)] {
        &self.observations
    }

    pub fn extend_observations(&mut self, with: Vec<(LiteralSource, Literal)>) {
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
