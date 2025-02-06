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

/// Information regarding the solve callback.
pub struct IpasirCallbacks {
    /// Version.
    version: u8,

    pub ipasir_terminate_callback: Option<extern "C" fn(data: *mut c_void) -> c_int>,
    pub ipasir_terminate_data: *mut c_void,

    pub ipasir_addition_callback: Option<extern "C" fn(data: *mut c_void, clause: *mut i32)>,
    pub ipasir_addition_callback_length: u32,
    pub ipasir_addition_data: *mut c_void,
}

impl IpasirCallbacks {
    /// Calls the IPASIR addition callback, if defined.
    ///
    /// # Safety
    /// The IPASIR API requires a pointer to the clause.
    /// And, transmute is used to transmute a const pointer to a mutable pointer, if integer literals are used.
    /// Safety can be restored by copying the clause, though this seems excessive.
    /// Though, regardless of this, the method calls an external C function.
    #[allow(clippy::useless_conversion)]
    pub unsafe fn call_ipasir_addition_callback(&self, clause: &CClause) {
        if let Some(addition_callback) = self.ipasir_addition_callback {
            if clause.size() <= self.ipasir_addition_callback_length as usize {
                if cfg!(feature = "boolean") {
                    let mut int_clause: IntClause =
                        clause.literals().map(|literal| literal.into()).collect();
                    let callback_ptr: *mut i32 = int_clause.as_mut_ptr();

                    addition_callback(self.ipasir_addition_data, callback_ptr);
                } else {
                    let clause_ptr: *const i32 = clause.as_ptr();
                    let callback_ptr: *mut i32 = unsafe { std::mem::transmute(clause_ptr) };
                    addition_callback(self.ipasir_addition_data, callback_ptr);
                };
            }
        }
    }

    /// # Safety
    /// Calls an external C function.
    pub unsafe fn call_ipasir_terminate_callback(&self) -> i32 {
        if let Some(terminate_callback) = self.ipasir_terminate_callback {
            terminate_callback(self.ipasir_terminate_data)
        } else {
            1
        }
    }
}

impl Default for IpasirCallbacks {
    fn default() -> Self {
        Self {
            version: 1,
            ipasir_terminate_callback: None,
            ipasir_terminate_data: std::ptr::dangling_mut(),

            ipasir_addition_callback: None,
            ipasir_addition_callback_length: 0,
            ipasir_addition_data: std::ptr::dangling_mut(),
        }
    }
}
