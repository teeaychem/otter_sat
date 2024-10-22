mod analysis;
pub mod builder;
pub mod config;
pub mod core;
mod resolution_buffer;
pub mod store;

use {
    crate::context::config::Config,
    crate::io::ContextWindow,
    crate::structures::{level::Level, literal::Literal, variable::delegate::VariableStore},
    store::{ClauseId, ClauseKey, ClauseStore},
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
    clause_store: ClauseStore,
    variables: VariableStore,
    pub time: Duration,
    config: config::Config,
    implication_graph: ResolutionGraph,
    pub window: Option<ContextWindow>,
    pub status: Status,
}

#[derive(Debug)]
pub enum Status {
    Initialised,
    AssertingClause(ClauseKey),
    MissedImplication(ClauseKey),
    NoSolution(ClauseKey),
    ChoiceMade,
    AllAssigned,
}

impl Context {
    // pub fn from_formula(formula: Formula, config: config::Config) -> Self {

    //     println!("c Parsing formula from file: {:?}", config.formula_file);
    //     println!(
    //         "c Parsed formula with {} variables and {} clauses",
    //         formula.variable_count(),
    //         formula.clause_count()
    //     );

    //     let mut the_context = Self::with_size_hints(formula.variable_count(), formula.clause_count(), config);

    //     for formula_clause in formula.clauses {
    //         assert!(
    //             !formula_clause.is_empty(),
    //             "c The formula contains an empty clause"
    //         );

    //         match formula_clause.len() {
    //             1 => {
    //                 let formula_literal = formula_clause.first().expect("literal vanish");
    //                 let the_literal = self.literal_ensure(formula_literal.name(), polarity);

    //                 the_context.literal_from_string(formula_clause.first());

    //                 the_context.literal_update(
    //                     *formula_clause.first().expect("literal vanish"),
    //                     0,
    //                     LiteralSource::Assumption,
    //                 );
    //             }
    //             _ => {
    //                 the_context.store_clause(formula_clause, Source::Formula);
    //             }
    //         }
    //     }

    //     the_context
    // }

    pub fn with_size_hints(variable_count: usize, clause_count: usize, config: Config) -> Self {
        let the_window = match config.show_stats {
            true => Some(ContextWindow::new(&config)),
            false => None,
        };

        let mut the_context = Self {
            conflicts: 0,
            conflicts_since_last_forget: 0,
            conflicts_since_last_reset: 0,
            iterations: 0,
            levels: Vec::<Level>::with_capacity(variable_count),
            restarts: 0,
            clause_store: ClauseStore::with_capacity(clause_count),
            variables: VariableStore::with_capactiy(variable_count),
            time: Duration::new(0, 0),
            config,
            implication_graph: Graph::new(),
            window: the_window,
            status: Status::Initialised,
        };
        the_context.levels.push(Level::new(0));
        the_context
    }
}

impl Default for Context {
    fn default() -> Self {
        let mut the_context = Context {
            conflicts: 0,
            conflicts_since_last_forget: 0,
            conflicts_since_last_reset: 0,
            iterations: 0,
            levels: Vec::<Level>::with_capacity(1024),
            restarts: 0,
            clause_store: ClauseStore::new(),
            variables: VariableStore::default(),
            time: Duration::new(0, 0),
            config: Config::default(),
            implication_graph: Graph::default(),
            window: None,
            status: Status::Initialised,
        };
        the_context.levels.push(Level::new(0));
        the_context
    }
}
