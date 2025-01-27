use std::collections::{HashMap, HashSet};

use crate::{
    config::Config,
    context::Context,
    db::ClauseKey,
    structures::{atom::Atom, clause::cClause, literal::cLiteral},
};

pub mod ipasir_one;
pub mod ipasir_two;

const IPASIR_SIGNATURE: &std::ffi::CStr = c"otter_sat 0.0.10";

/// A struct which bundles a context with structures to support distinct external representations of atoms.
///
/// Required, as a context does not support the creation of arbitrary atoms.
pub struct ContextBundle {
    /// A context.
    context: Context,

    /// A map from the external atom to a context atom.
    ei_map: HashMap<u32, Atom>,

    /// A map from the context atom to it's external representation.
    ///
    /// Here, the external representation is accessed by using the context atom as an index to the vector.
    ie_map: Vec<u32>,

    /// A buffer for the creation of a clause.
    clause_buffer: cClause,

    /// The keys to a an unsatisfiable core of the formula.
    core_keys: Vec<ClauseKey>,

    /// The literals which occur in the unsatisfiable core identified by [core_keys].
    core_literals: HashSet<cLiteral>,
}

impl Default for ContextBundle {
    fn default() -> Self {
        ContextBundle {
            context: Context::from_config(Config::default(), None),
            ei_map: HashMap::default(),
            ie_map: vec![0],
            clause_buffer: Vec::default(),
            core_keys: Vec::default(),
            core_literals: HashSet::default(),
        }
    }
}
