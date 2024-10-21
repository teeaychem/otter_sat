mod analysis;
pub mod config;
pub mod core;
mod resolution_buffer;
pub mod store;

use crate::{
    context::store::{ClauseId, ClauseKey, ClauseStore},
    structures::{
        clause::stored::Source,
        formula::Formula,
        level::Level,
        literal::{Literal, Source as LiteralSource},
        variable::delegate::VariableStore,
    },
    ContextWindow,
};

use petgraph::Graph;
use std::time::Duration;

#[derive(Debug)]
pub enum ImplicationGraphNode {
    Clause(GraphClause),
    Literal(GraphLiteral),
}

#[derive(Debug)]
pub struct GraphClause {
    clause_id: ClauseId,
    key: ClauseKey,
}

#[derive(Debug)]
pub struct GraphLiteral {
    literal: Literal,
}

pub type ResolutionGraph = Graph<ImplicationGraphNode, ()>;

pub struct Context {
    conflicts: usize,
    conflicts_since_last_forget: usize,
    conflicts_since_last_reset: usize,
    iterations: usize,
    levels: Vec<Level>,
    restarts: usize,
    stored_clauses: ClauseStore,
    variables: VariableStore,
    pub time: Duration,
    config: config::Config,
    implication_graph: ResolutionGraph,
    pub window: Option<ContextWindow>,
}

pub enum Status {
    AssertingClause,
    MissedImplication,
    NoSolution,
}

pub enum Result {
    Satisfiable,
    Unsatisfiable(ClauseKey),
    Unknown,
}

impl Context {
    pub fn from_formula(
        formula: Formula,
        config: config::Config,
        window: Option<ContextWindow>,
    ) -> Self {
        let variables = formula.variables;
        let clauses = formula.clauses;
        let variable_count = variables.len();

        let mut the_context = Self {
            conflicts: 0,
            conflicts_since_last_forget: 0,
            conflicts_since_last_reset: 0,
            iterations: 0,
            levels: Vec::<Level>::with_capacity(variable_count),
            restarts: 0,
            stored_clauses: ClauseStore::new(),
            variables: VariableStore::new(variables),
            time: Duration::new(0, 0),
            config,
            implication_graph: Graph::new(),
            window,
        };
        the_context.levels.push(Level::new(0));

        for formula_clause in clauses {
            assert!(
                !formula_clause.is_empty(),
                "c The formula contains an empty clause"
            );

            match formula_clause.len() {
                1 => {
                    the_context.literal_update(
                        *formula_clause.first().expect("literal vanish"),
                        0,
                        LiteralSource::Assumption,
                    );
                }
                _ => {
                    the_context.store_clause(formula_clause, Source::Formula);
                }
            }
        }

        the_context
    }
}
