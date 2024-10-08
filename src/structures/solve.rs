mod analysis;
pub mod clause_store;
pub mod config;
pub mod core;
mod stats;
mod the_solve;

use crate::structures::{level::Level, literal::Literal, variable::Variable};
use clause_store::ClauseStore;

use std::collections::VecDeque;

pub struct Solve {
    conflicts: usize,
    conflicts_since_last_forget: usize,
    conflicts_since_last_reset: usize,
    restarts: usize,
    pub variables: Vec<Variable>,
    pub valuation: Vec<Option<bool>>,
    pub levels: Vec<Level>,
    pub clauses_stored: ClauseStore,
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
