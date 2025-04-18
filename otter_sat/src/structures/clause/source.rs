/// The source of a clause.
#[derive(Clone, Copy)]
pub enum ClauseSource {
    /// A *unit* clause obtained via BCP.
    BCP,

    /// A *unit* clause set by free decision on the value of the contained atom.
    Unit,

    /// A clause read from a formula.
    Original,

    /// A clause derived via resolution (during analysis, etc.)
    Resolution,
}
