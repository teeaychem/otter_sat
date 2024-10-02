mod analysis;
pub mod config;
pub mod core;
mod the_solve;
mod stats;

use crate::structures::{
    formula::Formula, level::Level, literal::Literal, clause::stored_clause::StoredClause,
    variable::Variable,
};
use std::collections::VecDeque;
use std::rc::Rc;

#[derive(Debug)]
pub struct Solve<'formula> {
    _formula: &'formula Formula,
    pub conflicts: usize,
    pub conflcits_since_last_forget: usize,
    pub forgets: usize,
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
