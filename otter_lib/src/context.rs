use crate::{
    config::context::Config,
    db::{clause::ClauseDB, consequence_q::ConsequenceQ, literal::LiteralDB, variable::VariableDB},
    dispatch::{
        library::report::{self},
        Dispatch,
    },
    misc::random::MinimalPCG32,
    types::gen::Solve,
};

use crossbeam::channel::Sender;
use rand::SeedableRng;

// pub type RngChoice = rand::rngs::mock::StepRng;

use std::time::Duration;

pub struct Counters {
    pub conflicts: usize,
    pub conflicts_in_memory: usize,
    pub choices: usize,
    pub iterations: usize,
    pub restarts: usize,
    pub time: Duration,
    pub luby: crate::generic::luby::Luby,
    pub rng: MinimalPCG32,
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

            rng: MinimalPCG32::from_seed(0_u64.to_le_bytes()),
        }
    }
}

/// The context
pub struct Context {
    pub config: Config,

    pub counters: Counters,

    pub clause_db: ClauseDB,
    pub variable_db: VariableDB,
    pub literal_db: LiteralDB,

    pub status: Solve,
    pub tx: Option<Sender<Dispatch>>, //
    pub consequence_q: ConsequenceQ,
}

impl Context {
    pub fn from_config(config: Config, tx: Option<Sender<Dispatch>>) -> Self {
        Self {
            counters: Counters::default(),
            literal_db: LiteralDB::new(tx.clone()),
            clause_db: ClauseDB::new(&config, tx.clone()),
            variable_db: VariableDB::new(&config, tx.clone()),
            config,
            status: Solve::Initialised,
            tx,
            consequence_q: ConsequenceQ::default(),
        }
    }
}

impl Context {
    pub fn report(&self) -> report::Solve {
        match self.status {
            Solve::FullValuation => report::Solve::Satisfiable,
            Solve::NoClauses => report::Solve::Satisfiable,
            Solve::NoSolution => report::Solve::Unsatisfiable,
            _ => report::Solve::Unknown,
        }
    }
}

impl std::fmt::Display for Solve {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Solve::Initialised => write!(f, "Initialised"),
            Solve::AssertingClause => write!(f, "AssertingClause"),
            Solve::NoSolution => write!(f, "NoSolution"),
            Solve::ChoiceMade => write!(f, "ChoiceMade"),
            Solve::FullValuation => write!(f, "AllAssigned"),
            Solve::NoClauses => write!(f, "NoClauses"),
            Solve::Proof => write!(f, "Proof"),
        }
    }
}
