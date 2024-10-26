mod analysis;
pub mod builder;
pub mod core;
mod resolution_buffer;
pub mod store;

const RNG_SEED: u64 = 0;

use {
    crate::{
        config::{self, Config},
        io::window::ContextWindow,
        structures::{level::Level, literal::Literal, variable::delegate::VariableStore},
    },
    store::{ClauseId, ClauseKey, ClauseStore},
};

use rand_xoshiro::{rand_core::SeedableRng, Xoroshiro128Plus};

// pub type RngChoice = rand::rngs::mock::StepRng;
pub type RngChoice = Xoroshiro128Plus;

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
    rng: RngChoice,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Status {
    Initialised,
    AssertingClause(ClauseKey),
    MissedImplication(ClauseKey),
    NoSolution(ClauseKey),
    ChoiceMade,
    AllAssigned,
}

impl Context {
    pub fn default_config(config: Config) -> Self {
        Self::with_size_hints(1024, 32768, config)
    }

    pub fn with_size_hints(variable_count: usize, clause_count: usize, config: Config) -> Self {
        let the_window = match config.show_stats {
            true => Some(ContextWindow::default()),
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
            rng: RngChoice::seed_from_u64(RNG_SEED), //RngChoice::new(0,1)
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
            clause_store: ClauseStore::default(),
            variables: VariableStore::default(),
            time: Duration::new(0, 0),
            config: Config::default(),
            implication_graph: Graph::default(),
            window: None,
            status: Status::Initialised,
            rng: RngChoice::seed_from_u64(RNG_SEED),
        };
        the_context.levels.push(Level::new(0));
        the_context
    }
}
