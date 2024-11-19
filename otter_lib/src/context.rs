use crate::{
    config::{
        defaults::{self},
        Config,
    },
    db::{clause::ClauseDB, consequence_q::ConsequenceQ, literal::LiteralDB, variable::VariableDB},
    dispatch::{
        library::report::{self},
        Dispatch,
    },
    structures::{clause::Clause, literal::LiteralT},
    types::gen::Solve,
};

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

    pub status: Solve,
    pub tx: Option<Sender<Dispatch>>, //
    pub consequence_q: ConsequenceQ,
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

impl Context {
    pub fn from_config(config: Config, tx: Option<Sender<Dispatch>>) -> Self {
        Self {
            counters: Counters::default(),
            literal_db: LiteralDB::new(tx.clone()),
            clause_db: ClauseDB::default(tx.clone(), &config),
            variable_db: VariableDB::new(tx.clone()),
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

    pub fn clause_database(&self) -> Vec<String> {
        self.clause_db
            .all_clauses()
            .map(|clause| clause.as_dimacs(&self.variable_db, true))
            .collect::<Vec<_>>()
    }

    pub fn proven_literal_database(&self) -> Vec<String> {
        self.literal_db
            .proven_literals()
            .iter()
            .map(|literal| {
                format!(
                    "{} 0",
                    self.variable_db.external_representation(literal.var())
                )
            })
            .collect::<Vec<_>>()
    }
}
