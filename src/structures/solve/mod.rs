mod analysis;
pub mod config;
pub mod core;
mod the_solve;
mod resolution_buffer;

use crate::structures::{
    level::{Level, LevelIndex},
    literal::Literal,
    variable::Variable,
    valuation::Valuation,
};

use crate::structures::clause::stored::StoredClause;
use slotmap::{DefaultKey, SlotMap};

use std::collections::VecDeque;
use std::time::Duration;

type ClauseStore = SlotMap<DefaultKey, StoredClause>;

pub struct Solve {
    time: Duration,
    iterations: usize,
    conflicts: usize,
    conflicts_since_last_forget: usize,
    conflicts_since_last_reset: usize,
    restarts: usize,
    variables: Vec<Variable>,
    valuation: Box<[Option<bool>]>,
    levels: Vec<Level>,
    formula_clauses: ClauseStore,
    learnt_clauses: ClauseStore,
    consequence_q: VecDeque<Literal>,
}

pub enum Status {
    AssertingClause,
    MissedImplication,
    NoSolution,
}

pub enum Result {
    Satisfiable,
    Unsatisfiable,
    Unknown,
}

pub fn retreive<'a>(
    formula: &'a ClauseStore,
    learnt: &'a ClauseStore,
    key: ClauseKey,
) -> Option<&'a StoredClause> {
    match key {
        ClauseKey::Formula(key) => formula.get(key),
        ClauseKey::Learnt(key) => learnt.get(key),
    }
}

pub fn retreive_unsafe<'a>(
    formula: &'a ClauseStore,
    learnt: &'a ClauseStore,
    key: ClauseKey,
) -> &'a StoredClause {
    match key {
        ClauseKey::Formula(key) => unsafe { formula.get_unchecked(key) },
        ClauseKey::Learnt(key) => unsafe { learnt.get_unchecked(key) },
    }
}

pub fn retreive_mut<'a>(
    formula: &'a mut ClauseStore,
    learnt: &'a mut ClauseStore,
    key: ClauseKey,
) -> Option<&'a mut StoredClause> {
    match key {
        ClauseKey::Formula(key) => formula.get_mut(key),
        ClauseKey::Learnt(key) => learnt.get_mut(key),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ClauseKey {
    Formula(slotmap::DefaultKey),
    Learnt(slotmap::DefaultKey),
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

    pub fn variables(&self) -> &[Variable] {
        &self.variables
    }

    pub fn valuation(&self) -> &impl Valuation {
        &self.valuation
    }
}

impl Solve {
    pub fn display_stats(&self) {
        println!("c STATS");
        println!("c   ITERATIONS      {}", self.iterations);
        println!("c   CONFLICTS       {}", self.conflicts);
        println!(
            "c   CONFLICT RATIO  {:.4?}",
            self.conflicts as f32 / self.iterations as f32
        );
        println!("c   TIME            {:.2?}", self.time);
    }
}
