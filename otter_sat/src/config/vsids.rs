/*!
Ways to apply VSIDS (variable state independent decay sum) during during resolution-based analysis.

See [Understanding VSIDS branching heuristics in conflict-driven clause-learning sat solvers](https://arxiv.org/abs/1506.08905) for an overview of VSIDS .
*/

/// Supported VSIDS variants.
#[derive(Clone, Copy)]
#[allow(clippy::upper_case_acronyms)]
pub enum VSIDS {
    /// A variant which mimics the VSIDS used by [Chaff](https://dl.acm.org/doi/10.1145/378239.379017).\
    /// When learning a clause by applying resolution to a sequence of clauses every atom occurring in the learnt clause is bumped.
    Chaff,
    /// A variant which mimics the VSIDS used by [MiniSAT](https://link.springer.com/chapter/10.1007/978-3-540-24605-3_37).\
    /// When learning a clause by applying resolution to a sequence of clauses every atom occurring in some clause used during resolution (including the learnt clause) is bumped.
    MiniSAT,
}

impl std::fmt::Display for VSIDS {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Chaff => write!(f, "Chaff"),
            Self::MiniSAT => write!(f, "MiniSAT"),
        }
    }
}
