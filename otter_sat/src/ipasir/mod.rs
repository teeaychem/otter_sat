use std::{
    collections::{HashMap, HashSet},
    ffi::{c_int, c_void},
};

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
            clause_buffer: Vec::default(),
            core_keys: Vec::default(),
            core_literals: HashSet::default(),
        }
    }
}

pub struct IpasirSolveCallbacks {
    pub ipasir_terminate_callback: Option<extern "C" fn(data: *mut c_void) -> c_int>,
    pub ipasir_terminate_data: *mut c_void,
}

pub struct IpasirClauseDBCallbacks {
    pub ipasir_addition_callback: Option<extern "C" fn(data: *mut c_void, clause: *mut i32)>,
    pub ipasir_addition_callback_length: u32,
    pub ipasir_addition_data: *mut c_void,
}

impl Default for IpasirSolveCallbacks {
    fn default() -> Self {
        Self {
            ipasir_terminate_callback: None,
            ipasir_terminate_data: std::ptr::dangling_mut(),
        }
    }
}

impl Default for IpasirClauseDBCallbacks {
    fn default() -> Self {
        Self {
            ipasir_addition_callback: None,
            ipasir_addition_callback_length: 0,
            ipasir_addition_data: std::ptr::dangling_mut(),
        }
    }
}
