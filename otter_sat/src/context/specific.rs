use crate::{
    config::Config,
    db::{atom::AtomDB, clause::ClauseDB, watches::Watches},
    generic::random::MinimalPCG32,
    resolution_buffer::ResolutionBuffer,
};

use rand::SeedableRng;

use super::{ContextState, Counters, GenericContext};

/// A context which uses [MinimalPCG32] as a source of randomness.
pub type Context = GenericContext<MinimalPCG32>;

impl Context {
    /// Creates a context from some given configuration.
    pub fn from_config(config: Config) -> Self {
        let mut ctx = Self {
            atom_db: AtomDB::new(&config),
            watch_dbs: Watches::default(),
            clause_db: ClauseDB::new(&config),
            resolution_buffer: ResolutionBuffer::new(&config),

            config,

            counters: Counters::default(),

            rng: crate::generic::random::MinimalPCG32::from_seed(0_u64.to_le_bytes()),
            state: ContextState::Configuration,

            callback_terminate: None,
        };
        ctx.init();
        ctx
    }
}
