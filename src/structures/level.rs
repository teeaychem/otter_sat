use std::fmt::Debug;

use crate::structures::{
    literal::{Literal, LiteralSource},
    solve::Solve,
};

pub type LevelIndex = usize;

#[derive(Debug)]
pub struct Level {
    index: LevelIndex,
    pub choice: Option<Literal>,
    pub observations: Vec<(LiteralSource, Literal)>,
}

impl Level {
    pub fn new(index: LevelIndex) -> Self {
        Level {
            index,
            choice: None,
            observations: vec![],
        }
    }

    pub fn index(&self) -> LevelIndex {
        self.index
    }

    pub fn record_literal(&mut self, literal: Literal, source: &LiteralSource) {
        match source {
            LiteralSource::Choice => self.choice = Some(literal),
            LiteralSource::HobsonChoice
            | LiteralSource::Assumption
            | LiteralSource::Resolution(_)
            | LiteralSource::StoredClause(_) => self.observations.push((source.clone(), literal)),
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
                .cloned(),
        )
    }
}

impl Solve {
    pub fn add_fresh_level(&mut self) -> LevelIndex {
        let index = self.levels.len();
        let the_level = Level::new(index);
        self.levels.push(the_level);
        index
    }

    pub fn level(&self) -> &Level {
        let index = self.levels.len() - 1;
        &self.levels[index]
    }
}
