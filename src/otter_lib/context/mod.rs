mod analysis;
pub mod builder;
pub mod core;
mod preprocessing;
pub mod reports;
mod resolution_buffer;
pub mod solve;
pub mod stores;
pub mod unique_id;

use crate::{dispatch::Dispatch, types::gen::SolveStatus};

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

use crossbeam::channel::Sender;
use rand_xoshiro::{rand_core::SeedableRng, Xoroshiro128Plus};
use stores::level::LevelStore;

// pub type RngChoice = rand::rngs::mock::StepRng;
pub type RngChoice = Xoroshiro128Plus;

use std::{fmt::Debug, time::Duration};

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

pub struct Context {
    counters: Counters,
    pub levels: LevelStore,
    pub clause_store: ClauseStore,
    pub variables: VariableStore,
    pub config: config::Config,
    window: Option<ContextWindow>,
    pub status: SolveStatus,

    pub sender: Sender<Dispatch>, //
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
    pub fn from_config(config: Config, sender: Sender<Dispatch>) -> Self {
        let the_window = match config.io.show_stats {
            true => Some(ContextWindow::default()),
            false => None,
        };

        Self {
            counters: Counters::default(),
            levels: LevelStore::default(),
            clause_store: ClauseStore::default(&sender),
            variables: VariableStore::new(sender.clone()),
            config,
            window: the_window,
            status: SolveStatus::Initialised,
            sender,
        }
    }
}

impl Default for Context {
    fn default() -> Self {
        let (sender, _) = crossbeam::channel::bounded::<Dispatch>(0);
        Context {
            counters: Counters::default(),
            levels: LevelStore::default(),
            clause_store: ClauseStore::default(&sender),
            variables: VariableStore::new(sender.clone()),
            config: Config::default(),
            window: None,
            status: SolveStatus::Initialised,
            sender,
        }
    }
}
