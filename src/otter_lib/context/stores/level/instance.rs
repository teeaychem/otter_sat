use std::borrow::Borrow;

use crate::{
    context::stores::{level::Level, LevelIndex},
    structures::literal::{Literal, LiteralSource, LiteralTrait},
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

    pub fn record_literal<L: Borrow<impl LiteralTrait> + Copy>(
        &mut self,
        literal: L,
        source: LiteralSource,
    ) {
        match source {
            LiteralSource::Choice => self.choice = Some(literal.borrow().canonical()),
            LiteralSource::Pure
            | LiteralSource::Assumption
            | LiteralSource::Resolution(_)
            | LiteralSource::Analysis(_)
            | LiteralSource::BCP(_)
            | LiteralSource::Missed(_) => self
                .observations
                .push((source, literal.borrow().canonical())),
        }
    }

    pub fn observations(&self) -> &[(LiteralSource, Literal)] {
        &self.observations
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
