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
    pub watch_q: VecDeque<Literal>,
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

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ClauseKey {
    Formula(slotmap::DefaultKey),
    Learnt(slotmap::DefaultKey),
}
