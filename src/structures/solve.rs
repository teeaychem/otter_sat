mod analysis;
pub mod config;
pub mod core;
mod stats;
mod the_solve;

use crate::structures::{
    clause::stored_clause::StoredClause, level::Level, literal::Literal, variable::Variable,
};
use slotmap::{DefaultKey, SlotMap};
use std::collections::VecDeque;

use super::clause::stored_clause::ClauseKey;

pub struct Solve {
    conflicts: usize,
    conflicts_since_last_forget: usize,
    forgets: usize,
    pub variables: Vec<Variable>,
    pub valuation: Vec<Option<bool>>,
    pub levels: Vec<Level>,
    pub stored_clauses: ClauseStore,
    pub watch_q: VecDeque<Literal>,
}

pub struct ClauseStore {
    pub formula_clauses: SlotMap<DefaultKey, StoredClause>,
    pub learnt_clauses: SlotMap<DefaultKey, StoredClause>,
}

impl ClauseStore {

}

#[derive(Debug, PartialEq)]
pub enum SolveStatus {
    AssertingClause,
    Backtracked,
    NoSolution,
}

pub enum SolveResult {
    Satisfiable,
    Unsatisfiable,
    Unknown,
}

pub fn retreive(store: &ClauseStore, key: ClauseKey) -> &StoredClause {
        match key {
            ClauseKey::Formula(key) => &store.formula_clauses[key],
            ClauseKey::Learnt(key) => &store.learnt_clauses[key],
        }
    }
