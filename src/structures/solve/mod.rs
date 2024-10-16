mod analysis;
pub mod config;
mod resolution_buffer;
pub mod solve_core;
pub mod store;
mod the_solve;

use crate::structures::{
    clause::stored::Source,
    formula::Formula,
    level::Level,
    literal::{Literal, Source as LiteralSource},
    solve::store::ClauseStore,
    variable::Variable,
};

use std::{collections::VecDeque, time::Duration};

pub struct Solve {
    conflicts: usize,
    conflicts_since_last_forget: usize,
    conflicts_since_last_reset: usize,
    consequence_q: VecDeque<Literal>,
    iterations: usize,
    levels: Vec<Level>,
    restarts: usize,
    stored_clauses: ClauseStore,
    valuation: Box<[Option<bool>]>,
    variables: Vec<Variable>,
    pub time: Duration,
    config: config::Config
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

impl Solve {
    pub fn from_formula(formula: Formula, config: config::Config) -> Self {
        let variables = formula.variables;
        let clauses = formula.clauses;
        let variable_count = variables.len();

        let mut the_solve = Self {
            conflicts: 0,
            conflicts_since_last_forget: 0,
            conflicts_since_last_reset: 0,
            consequence_q: VecDeque::with_capacity(variable_count),
            iterations: 0,
            levels: Vec::<Level>::with_capacity(variable_count),
            restarts: 0,
            stored_clauses: ClauseStore::new(),
            valuation: vec![None; variables.len()].into_boxed_slice(),
            variables,
            time: Duration::new(0, 0),
            config
        };
        the_solve.levels.push(Level::new(0));

        for formula_clause in clauses {
            assert!(
                !formula_clause.is_empty(),
                "c The formula contains an empty clause"
            );

            match formula_clause.len() {
                1 => {
                    the_solve.literal_update(
                        *formula_clause.first().expect("Literal vanish"),
                        &LiteralSource::Assumption,
                    );
                }
                _ => {
                    the_solve.store_clause(formula_clause, Source::Formula);
                }
            }
        }

        the_solve
    }
}
