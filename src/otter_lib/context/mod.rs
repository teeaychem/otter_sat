mod analysis;
pub mod builder;
pub mod core;
pub mod reports;
mod resolution_buffer;
pub mod store;

use {
    crate::{
        config::{
            self,
            defaults::{self},
            Config,
        },
        io::window::ContextWindow,
        structures::{level::Level, literal::Literal, variable::delegate::VariableStore},
    },
    store::{ClauseKey, ClauseStore},
};

use rand_xoshiro::{rand_core::SeedableRng, Xoroshiro128Plus};

// pub type RngChoice = rand::rngs::mock::StepRng;
pub type RngChoice = Xoroshiro128Plus;

use std::time::Duration;

pub struct Counters {
    pub conflicts: usize,
    pub conflicts_since_last_forget: usize,
    pub conflicts_since_last_reset: usize,
    pub decisions: usize,
    pub iterations: usize,
    pub restarts: usize,
    pub time: Duration,
}

impl Default for Counters {
    fn default() -> Self {
        Counters {
            conflicts_since_last_forget: 0,
            conflicts_since_last_reset: 0,
            decisions: 0,
            iterations: 0,
            restarts: 0,
            time: Duration::from_secs(0),
            conflicts: 0,
        }
    }
}

pub struct Context {
    counters: Counters,
    levels: Vec<Level>,
    clause_store: ClauseStore,
    variables: VariableStore,
    config: config::Config,
    pub window: Option<ContextWindow>,
    pub status: Status,
    rng: RngChoice,
    pub proofs: Vec<(Literal, Vec<ClauseKey>)>,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Status {
    Initialised,
    AssertingClause(ClauseKey),
    MissedImplication(ClauseKey),
    NoSolution(ClauseKey),
    ChoiceMade,
    AllAssigned,
    NoClauses,
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum Report {
    Satisfiable,
    Unsatisfiable,
    Unknown,
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
            counters: Counters::default(),
            levels: Vec::<Level>::with_capacity(variable_count),
            clause_store: ClauseStore::with_capacity(clause_count),
            variables: VariableStore::with_capactiy(variable_count),
            config,
            window: the_window,
            status: Status::Initialised,
            rng: RngChoice::seed_from_u64(defaults::RNG_SEED), //RngChoice::new(0,1)
            proofs: Vec::new(),
        };
        the_context.levels.push(Level::new(0));
        the_context
    }
}

impl Default for Context {
    fn default() -> Self {
        let mut the_context = Context {
            counters: Counters::default(),
            levels: Vec::<Level>::with_capacity(1024),
            clause_store: ClauseStore::default(),
            variables: VariableStore::default(),
            config: Config::default(),
            window: None,
            status: Status::Initialised,
            rng: RngChoice::seed_from_u64(defaults::RNG_SEED),
            proofs: Vec::new(),
        };
        the_context.levels.push(Level::new(0));
        the_context
    }
}
