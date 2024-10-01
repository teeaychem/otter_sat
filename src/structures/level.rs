use std::fmt::Debug;

use crate::structures::{solve::Solve, Literal, LiteralSource, VariableId};

pub type LevelIndex = usize;

#[derive(Clone, Debug)]
pub struct Level {
    index: LevelIndex,
    choice: Option<Literal>,
    observations: Vec<(LiteralSource, Literal)>,
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

    pub fn get_choice(&self) -> Option<Literal> {
        self.choice
    }

    pub fn record_literal(&mut self, literal: Literal, source: &LiteralSource) {
        match source {
            LiteralSource::Choice => {
                if self.choice.is_some() {
                    panic!("Attempting to make multiple choices on a single level")
                }
                self.choice = Some(literal);
            }
            LiteralSource::HobsonChoice
            | LiteralSource::Assumption
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

    pub fn variables(&self) -> impl Iterator<Item = VariableId> + '_ {
        self.choice
            .into_iter()
            .map(|l| l.v_id)
            .chain(self.observations.iter().map(|(_, l)| l.v_id))
    }
}

impl<'borrow, 'solve> Solve<'solve> {
    pub fn add_fresh_level(&'borrow mut self) -> LevelIndex {
        let index = self.levels.len();
        let the_level = Level::new(index);
        self.levels.push(the_level);
        index
    }
}

impl<'borrow, 'level, 'solve: 'level> Solve<'solve> {
    pub fn top_level(&'borrow self) -> &Level {
        &self.levels[0]
    }

    pub fn top_level_mut(&'borrow mut self) -> &mut Level {
        &mut self.levels[0]
    }

    pub fn current_level(&'borrow self) -> &Level {
        let index = self.levels.len() - 1;
        &self.levels[index]
    }

    pub fn current_level_mut(&'borrow mut self) -> &mut Level {
        let index = self.levels.len() - 1;
        &mut self.levels[index]
    }
}
