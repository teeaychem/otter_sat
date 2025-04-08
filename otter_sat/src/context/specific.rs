use crate::{
    atom_cells::AtomCells,
    config::Config,
    db::{clause::ClauseDB, trail::Trail, watches::Watches},
    generic::{index_heap::IndexHeap, random::MinimalPCG32},
};

use rand::SeedableRng;

use super::{ContextState, Counters, GenericContext};

/// A context which uses [MinimalPCG32] as a source of randomness.
pub type Context = GenericContext<MinimalPCG32>;

impl Context {
    /// Creates a context from some given configuration.
    pub fn from_config(config: Config) -> Self {
        let mut ctx = Self {
            // valuation: CValuation::default(),
            atom_activity: IndexHeap::default(),
            watches: Watches::default(),
            clause_db: ClauseDB::new(&config),
            atom_cells: AtomCells::new(&config),
            trail: Trail::default(),

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
