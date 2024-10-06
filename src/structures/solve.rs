mod analysis;
pub mod config;
pub mod core;
mod the_solve;
mod stats;

use crate::structures::{
    level::Level, literal::Literal, clause::stored_clause::StoredClause,
    variable::Variable,
};
use std::collections::VecDeque;
use std::rc::Rc;

pub struct Solve {
    conflicts: usize,
    conflicts_since_last_forget: usize,
    forgets: usize,
    pub variables: Vec<Variable>,
    pub valuation: Vec<Option<bool>>,
    pub levels: Vec<Level>,
    pub formula_clauses: Vec<Rc<StoredClause>>,
    pub learnt_clauses: Vec<Rc<StoredClause>>,
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
