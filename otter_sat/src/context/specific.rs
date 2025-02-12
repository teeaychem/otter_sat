use crate::{
    config::Config,
    db::{atom::AtomDB, clause::ClauseDB, consequence_q::ConsequenceQ, literal::LiteralDB},
    dispatch::Dispatch,
    generic::minimal_pcg::MinimalPCG32,
    resolution_buffer::ResolutionBuffer,
};

use rand::SeedableRng;
use std::rc::Rc;

use super::{ContextState, Counters, GenericContext};

/// A context which uses [MinimalPCG32] as a source of randomness.
pub type Context = GenericContext<MinimalPCG32>;

impl Context {
    /// Creates a context from some given configuration.
    pub fn from_config(config: Config, dispatcher: Option<Rc<dyn Fn(Dispatch)>>) -> Self {
        Self {
            atom_db: AtomDB::new(&config, dispatcher.clone()),
            clause_db: ClauseDB::new(&config, dispatcher.clone()),
            literal_db: LiteralDB::new(&config, dispatcher.clone()),
            resolution_buffer: ResolutionBuffer::new(&config),

            config,

            consequence_q: ConsequenceQ::default(),
            counters: Counters::default(),

            rng: crate::generic::minimal_pcg::MinimalPCG32::from_seed(0_u64.to_le_bytes()),
            state: ContextState::Configuration,

            dispatcher,

            callback_terminate: None,
        }
    }
}
