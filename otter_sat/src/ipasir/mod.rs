use std::collections::HashMap;

use crate::{
    config::Config,
    context::Context,
    structures::{atom::Atom, clause::vClause},
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
    clause_buffer: vClause,
}

impl Default for ContextBundle {
    fn default() -> Self {
        ContextBundle {
            context: Context::from_config(Config::default(), None),
            ei_map: HashMap::default(),
            ie_map: vec![0],
            clause_buffer: Vec::default(),
        }
    }
}
