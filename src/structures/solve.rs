mod analysis;
pub mod clause_store;
pub mod config;
pub mod core;
mod stats;
mod the_solve;

use crate::structures::{level::Level, literal::Literal, variable::Variable};

use crate::structures::clause::stored_clause::StoredClause;
use slotmap::{DefaultKey, SlotMap};

use std::collections::VecDeque;

use super::literal::LiteralSource;

type ClauseStore = SlotMap<DefaultKey, StoredClause>;

pub struct Solve {
    conflicts: usize,
    conflicts_since_last_forget: usize,
    conflicts_since_last_reset: usize,
    restarts: usize,
    pub variables: Vec<Variable>,
    pub valuation: Vec<Option<bool>>,
    pub levels: Vec<Level>,
    pub formula_clauses: ClauseStore,
    pub learnt_clauses: ClauseStore,
    pub watch_q: VecDeque<(Literal, LiteralSource)>,
}

#[derive(Debug, PartialEq)]
pub enum SolveStatus {
    AssertingClause,
    NoSolution,
}

pub enum SolveResult {
    Satisfiable,
    Unsatisfiable,
    Unknown,
}

pub fn retreive<'a>(
    formula: &'a ClauseStore,
    learnt: &'a ClauseStore,
    key: ClauseKey,
) -> &'a StoredClause {
    match key {
        ClauseKey::Formula(key) => &formula[key],
        ClauseKey::Learnt(key) => &learnt[key],
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ClauseKey {
    Formula(slotmap::DefaultKey),
    Learnt(slotmap::DefaultKey),
}
