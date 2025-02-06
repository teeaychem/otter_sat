//! Bindings for the reentrant incremental sat solver API --- the IPASIR C IPA.
//!
//! The IPASIR C API defines a handful of functions used for incremental SAT solving.
//! This module provides bindings to the API.
//!
//! Specifically, this module includes a full implementation of bindings for the first version of the IPASIR API and template bindings for the second version of the API.
//!
//! Note, 'solver' and 'context' are synonymous in this module.\
//! Though, strictly, 'solver' is only used as, or when referring to, the parameter of an API function, and 'context' is only used to refer to an instance of the context structure.
//!
//! # Compiling a library
//!
//! By default, cargo does not build a library suitable for to linking to a C program.\
//! For details on building a suitable library, see: <https://doc.rust-lang.org/reference/linkage.html>
//!
//! # Efficiancy
//!
//! At present, the library uses a 'transparent' representation of literals added through the IPASIR API --- whether as part of a clause, as an assumption, etc.
//! This means if the literal -83 is added through the API all internal data structures will 'grow' to allow for 83 atoms.
//! In this respect, it is much more efficient to add a clause containing the largest literal first.

use std::{
    collections::{HashMap, HashSet},
    ffi::{c_int, c_void},
};

use crate::{
    config::Config,
    context::Context,
    db::ClauseKey,
    structures::{
        atom::Atom,
        clause::{CClause, Clause, IntClause},
        literal::CLiteral,
    },
};

mod callbacks;
pub use callbacks::IpasirCallbacks;

pub mod ipasir_one;
pub mod ipasir_two;

const IPASIR_SIGNATURE: &std::ffi::CStr = c"otter_sat 0.10.0";

/// A struct which bundles a context with some structures to the IPASIR API.
pub struct ContextBundle {
    /// A context.
    context: Context,

    /// A buffer to hold the literals of a clause being added to the solver.
    clause_buffer: CClause,

    /// The keys to a an unsatisfiable core of the formula.
    core_keys: Vec<ClauseKey>,

    /// The literals which occur in the unsatisfiable core identified by [core_keys].
    core_literals: HashSet<CLiteral>,
}

impl ContextBundle {
    /// Refreshes the bundled context and clears bundled structures if the context was not already fresh.
    pub fn keep_fresh(&mut self) {
        match self.context.refresh() {
            true => {
                self.core_keys.clear();
                self.core_literals.clear();
            }
            false => {}
        }
    }
}

impl Default for ContextBundle {
    fn default() -> Self {
        ContextBundle {
            context: Context::from_config(Config::default(), None),
            clause_buffer: Vec::default(),
            core_keys: Vec::default(),
            core_literals: HashSet::default(),
        }
    }
}
