pub mod builder;
pub mod consequence_q;
pub mod core;

pub mod reports;

use crate::{
    config::{defaults, Config},
    db::{clause::ClauseDB, literal::LiteralDB, variable::VariableDB},
    dispatch::Dispatch,
    types::gen::SolveStatus,
};

use consequence_q::ConsequenceQ;
use crossbeam::channel::Sender;
use rand_xoshiro::{rand_core::SeedableRng, Xoroshiro128Plus};

// pub type RngChoice = rand::rngs::mock::StepRng;
pub type RngChoice = Xoroshiro128Plus;

use std::time::Duration;

pub struct Counters {
    pub conflicts: usize,
    pub conflicts_in_memory: usize,
    pub choices: usize,
    pub iterations: usize,
    pub restarts: usize,
    pub time: Duration,
    pub luby: crate::generic::luby::Luby,
    pub rng: RngChoice,
}

impl Default for Counters {
    fn default() -> Self {
        Counters {
            conflicts_in_memory: 0,
            choices: 0,
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
    pub config: Config,

    pub counters: Counters,

    pub clause_db: ClauseDB,
    pub variable_db: VariableDB,
    pub literal_db: LiteralDB,

    pub status: SolveStatus,
    pub tx: Sender<Dispatch>, //
    pub consequence_q: ConsequenceQ,
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
    pub fn from_config(config: Config, tx: Sender<Dispatch>) -> Self {
        Self {
            counters: Counters::default(),
            literal_db: LiteralDB::new(tx.clone()),
            clause_db: ClauseDB::default(&tx, &config),
            variable_db: VariableDB::new(tx.clone()),
            config,
            status: SolveStatus::Initialised,
            tx,
            consequence_q: ConsequenceQ::default(),
        }
    }
}
