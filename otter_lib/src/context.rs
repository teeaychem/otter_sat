//! The context of a solve.

use crate::{
    config::Config,
    db::{clause::ClauseDB, consequence_q::ConsequenceQ, literal::LiteralDB, variable::VariableDB},
    dispatch::{
        library::report::{self},
        Dispatch,
    },
    types::gen::Solve,
};

use rand::SeedableRng;
use std::{rc::Rc, time::Duration};

pub struct Counters {
    pub conflicts: usize,
    pub fresh_conflicts: u32,
    pub choices: usize,
    pub iterations: usize,
    pub restarts: usize,
    pub time: Duration,
    pub luby: crate::generic::luby::Luby,
    pub rng: crate::generic::minimal_pcg::MinimalPCG32,
}

impl Default for Counters {
    fn default() -> Self {
        Counters {
            fresh_conflicts: 0,
            choices: 0,
            iterations: 0,
            restarts: 0,
            time: Duration::from_secs(0),
            conflicts: 0,

            luby: crate::generic::luby::Luby::default(),
            rng: crate::generic::minimal_pcg::MinimalPCG32::from_seed(0_u64.to_le_bytes()),
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

    pub consequence_q: ConsequenceQ,

    pub dispatcher: Option<Rc<dyn Fn(Dispatch)>>,

    pub status: Solve,
}

impl Context {
    pub fn from_config(config: Config, dispatcher: Option<Rc<dyn Fn(Dispatch)>>) -> Self {
        Self {
            counters: Counters::default(),
            literal_db: LiteralDB::new(dispatcher.clone()),
            clause_db: ClauseDB::new(&config, dispatcher.clone()),
            variable_db: VariableDB::new(&config, dispatcher.clone()),
            config,
            status: Solve::Initialised,
            consequence_q: ConsequenceQ::default(),
            dispatcher,
        }
    }
}

impl Context {
    pub fn report(&self) -> report::Solve {
        match self.status {
            Solve::FullValuation => report::Solve::Satisfiable,
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
            Solve::UnitClause => write!(f, "Proof"),
        }
    }
}
