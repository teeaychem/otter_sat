use crate::{
    config::Config,
    db::{
        atom::AtomDB, clause::ClauseDB, consequence_q::ConsequenceQ, dbStatus, literal::LiteralDB,
    },
    dispatch::Dispatch,
    generic::minimal_pcg::MinimalPCG32,
};

use rand::SeedableRng;
use std::rc::Rc;

use super::{Counters, GenericContext};

/// A context which uses [MinimalPCG32] as a source of randomness.
pub type Context = GenericContext<MinimalPCG32>;

impl Context {
    /// Creates a context from some given configuration.
    pub fn from_config(config: Config, dispatcher: Option<Rc<dyn Fn(Dispatch)>>) -> Self {
        Self {
            status: dbStatus::Unknown,

            counters: Counters::default(),

            literal_db: LiteralDB::new(dispatcher.clone()),
            clause_db: ClauseDB::new(&config, dispatcher.clone()),
            atom_db: AtomDB::new(&config, dispatcher.clone()),
            consequence_q: ConsequenceQ::default(),

            config,
            dispatcher,

            rng: crate::generic::minimal_pcg::MinimalPCG32::from_seed(0_u64.to_le_bytes()),
        }
    }
}
