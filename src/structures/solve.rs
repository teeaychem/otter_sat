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

pub struct Solve {
    conflicts: usize,
    conflicts_since_last_forget: usize,
    forgets: usize,
    pub variables: Vec<Variable>,
    pub valuation: Vec<Option<bool>>,
    pub levels: Vec<Level>,
    pub formula_clauses: SlotMap<DefaultKey, StoredClause>,
    pub learnt_clauses: SlotMap<DefaultKey, StoredClause>,
    pub watch_q: VecDeque<Literal>,
}

#[derive(Debug, PartialEq)]
pub enum SolveStatus {
    AssertingClause,
    Deduction(Literal),
    Backtracked,
    NoSolution,
}

pub enum SolveResult {
    Satisfiable,
    Unsatisfiable,
    Unknown,
}
