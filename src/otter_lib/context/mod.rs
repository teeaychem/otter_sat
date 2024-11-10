mod analysis;
pub mod builder;
pub mod core;
pub mod delta;
mod preprocessing;
pub mod reports;
mod resolution_buffer;
pub mod solve;
pub mod stores;
pub mod unique_id;

use crate::{types::gen::SolveStatus, FRAT::FRATProof};

use {
    crate::{
        config::{
            self,
            defaults::{self},
            Config,
        },
        io::window::ContextWindow,
    },
    stores::{clause::ClauseStore, variable::VariableStore},
};

use rand_xoshiro::{rand_core::SeedableRng, Xoroshiro128Plus};
use stores::level::LevelStore;
use unique_id::UniqueIdentifier;

// pub type RngChoice = rand::rngs::mock::StepRng;
pub type RngChoice = Xoroshiro128Plus;

use std::time::Duration;

pub struct Counters {
    pub conflicts: usize,
    pub conflicts_in_memory: usize,
    pub decisions: usize,
    pub iterations: usize,
    pub restarts: usize,
    pub time: Duration,
    luby: crate::generic::luby::Luby,
    rng: RngChoice,
}

impl Default for Counters {
    fn default() -> Self {
        Counters {
            conflicts_in_memory: 0,
            decisions: 0,
            iterations: 0,
            restarts: 0,
            time: Duration::from_secs(0),
            conflicts: 0,
            luby: crate::generic::luby::Luby::default(),
            rng: RngChoice::seed_from_u64(defaults::RNG_SEED), //RngChoice::new(0,1)
        }
    }
}

pub struct Traces {
    // TODO: Provide functions for serealising
    serial: Vec<(UniqueIdentifier, Vec<UniqueIdentifier>)>,
    frat: FRATProof,
}

pub struct Context {
    counters: Counters,
    pub levels: LevelStore,
    pub clause_store: ClauseStore,
    pub variables: VariableStore,
    pub config: config::Config,
    window: Option<ContextWindow>,
    status: SolveStatus,

    pub traces: Traces,
    //
}

impl std::fmt::Display for SolveStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SolveStatus::Initialised => write!(f, "Initialised"),
            SolveStatus::AssertingClause => write!(f, "AssertingClause"),
            SolveStatus::MissedImplication => write!(f, "MissedImplication"),
            SolveStatus::NoSolution => write!(f, "NoSolution"),
            SolveStatus::ChoiceMade => write!(f, "ChoiceMade"),
            SolveStatus::FullValuation => write!(f, "AllAssigned"),
            SolveStatus::NoClauses => write!(f, "NoClauses"),
            SolveStatus::Proof => write!(f, "Proof"),
        }
    }
}

impl Context {
    pub fn default_config(config: Config) -> Self {
        Self::with_size_hints(2048, 32768, config)
    }

    pub fn with_size_hints(variable_count: usize, clause_count: usize, config: Config) -> Self {
        let the_window = match config.io.show_stats {
            true => Some(ContextWindow::default()),
            false => None,
        };

        Self {
            counters: Counters::default(),
            levels: LevelStore::default(),
            clause_store: ClauseStore::with_capacity(clause_count),
            variables: VariableStore::with_capactiy(variable_count),
            config,
            window: the_window,
            status: SolveStatus::Initialised,
            traces: Traces {
                serial: Vec::default(),
                frat: FRATProof::new(),
            },
        }
    }
}

impl Default for Context {
    fn default() -> Self {
        Context {
            counters: Counters::default(),
            levels: LevelStore::default(),
            clause_store: ClauseStore::default(),
            variables: VariableStore::default(),
            config: Config::default(),
            window: None,
            status: SolveStatus::Initialised,
            traces: Traces {
                serial: Vec::default(),
                frat: FRATProof::new(),
            },
        }
    }
}
