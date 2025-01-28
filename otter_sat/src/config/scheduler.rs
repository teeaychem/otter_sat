/// Schedulers, for reduction of the clause database, etc.
///
/// Note: If two scheduled reductions coincide, only one reduction takes place.
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Scheduler {
    /// Reuce the clause database every `luby` times a luby interrupt happens.
    pub luby: Option<u32>,

    /// Reuce the clause database every `conflict` conflicts.
    pub conflict: Option<u32>,
}
