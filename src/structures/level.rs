use std::fmt::Debug;

use crate::structures::{
    ClauseId, Literal, LiteralSource, Solve, Valuation, ValuationError, ValuationVec, VariableId,
};

pub type LevelIndex = usize;

#[derive(Clone, Debug)]
pub struct Level {
    index: LevelIndex,
    choice: Option<Literal>,
    observations: Vec<Literal>,
    clauses_unit: Vec<(ClauseId, Literal)>,
}

impl Level {
    pub fn new(index: LevelIndex) -> Self {
        Level {
            index,
            choice: None,
            observations: vec![],
            clauses_unit: vec![],
        }
    }

    pub fn index(&self) -> LevelIndex {
        self.index
    }

    pub fn get_choice(&self) -> Option<Literal> {
        self.choice
    }

    pub fn record_literal(&mut self, literal: Literal, source: LiteralSource) {
        match source {
            LiteralSource::Choice => {
                if self.choice.is_some() {
                    panic!("Attempting to make multiple choices on a single level")
                }
                self.choice = Some(literal);
            }
            LiteralSource::HobsonChoice
            | LiteralSource::Assumption
            | LiteralSource::StoredClause(_)
            | LiteralSource::Deduced
            | LiteralSource::Conflict => self.observations.push(literal),
        }
    }

    pub fn literals(&self) -> impl Iterator<Item = Literal> + '_ {
        self.choice
            .into_iter()
            .chain(self.observations.iter().cloned())
    }

    pub fn variables(&self) -> impl Iterator<Item = VariableId> + '_ {
        self.choice
            .into_iter()
            .map(|l| l.v_id)
            .chain(self.observations.iter().map(|l| l.v_id))
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
    pub fn last_choice_level(&'borrow mut self) -> Option<&Level> {
        if self.levels.len() <= 1 {
            return None;
        }
        let the_level: Option<&Level> = self.levels.last();
        if let Some(level) = &the_level {
            self.valuation.clear_level(level);
        };

        the_level
    }

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

impl Solve<'_> {
    pub fn valuation_at(&self, level_index: LevelIndex) -> ValuationVec {
        let mut valuation = ValuationVec::new_for_variables(self.valuation.len());
        (0..=level_index).for_each(|i| {
            self.levels[i].literals().for_each(|l| {
                let _ = valuation.set_literal(l);
            })
        });
        valuation
    }
}
