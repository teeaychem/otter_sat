//! Partial bindings for IPASIR2 API.
//!
//! In particular:
//! - There is no support for options.\
//!   Support will be added when (or before) the API is finalised.
//! - There is no support for set_import.
//! - There is partial support for set_fixed.
//! - There is no support for noting that an added clause may be forgotten.
//!
//! Otherwise, if an IPASIR2 function wraps an IPASIR function, initial support is implemented.

use crate::{
    context::ContextState,
    dispatch::library::report::SolveReport,
    ipasir::{
        ipasir_one::{ipasir_failed, ipasir_init, ipasir_set_learn},
        ContextBundle, IpasirCallbacks, IPASIR_SIGNATURE,
    },
    structures::{
        clause::CClause,
        literal::{CLiteral, Literal},
    },
};

use super::ipasir_one::{
    ipasir_release, ipasir_set_terminate, ipasir_signature, ipasir_solve, ipasir_val,
};

use std::ffi::{c_char, c_int, c_void};

/// Codes used to indicate the success or failure of a function call.
#[allow(non_camel_case_types)]
#[repr(C)]
pub enum ipasir2_errorcode {
    IPASIR2_E_OK = 0,
    IPASIR2_E_UNKNOWN = 1,
    IPASIR2_E_UNSUPPORTED,
    IPASIR2_E_UNSUPPORTED_ARGUMENT,
    IPASIR2_E_UNSUPPORTED_OPTION,
    IPASIR2_E_INVALID_STATE,
    IPASIR2_E_INVALID_ARGUMENT,
    IPASIR2_E_INVALID_OPTION_VALUE,
}

/// States of the context, these are a subset of [ContextState].
#[allow(non_camel_case_types)]
#[repr(C)]
pub enum ipasir2_state {
    IPASIR2_S_CONFIG = 0,
    IPASIR2_S_INPUT = 1,
    IPASIR2_S_SAT,
    IPASIR2_S_UNSAT,
    IPASIR2_S_SOLVING,
}

/// IPASIR Configuration Options
#[allow(non_camel_case_types)]
#[repr(C)]
pub struct ipasir2_option {
    /// Unique option identifier.
    pub name: *const c_char,

    /// Minimum allowed value for the option.
    pub min: i64,

    /// Maximum allowed value for the option.
    pub max: i64,

    /// Maximal state in which the option may be set.
    pub max_state: ipasir2_state,

    /// Specifies if the option is eligible for use by automatic tuners.
    pub tunable: c_int,

    /// Specifies if the option may be set per variable.
    pub indexed: c_int,

    /// An opaque pointer for internal use in the setter function.
    pub handle: *const c_void,
}

/// Bind the name and version of this library to the given pointer.
/// # Safety
/// Writes the signature a raw pointer.
#[no_mangle]
pub unsafe extern "C" fn ipasir2_signature(signature: *mut *const c_char) -> ipasir2_errorcode {
    std::ptr::write(signature, ipasir_signature());

    ipasir2_errorcode::IPASIR2_E_OK
}

/// Initialises a solver a binds the given pointer to its address.
/// # Safety
/// Releases the initialised solver to a raw pointer.
#[no_mangle]
pub unsafe extern "C" fn ipasir2_init(solver: *mut *mut c_void) -> ipasir2_errorcode {
    std::ptr::write(solver, ipasir_init());

    ipasir2_errorcode::IPASIR2_E_OK
}

/// Releases the bound solver, so long as it is not solving.
/// # Safety
/// Recovers a context bundle from a raw pointer.
#[no_mangle]
pub unsafe extern "C" fn ipasir2_release(solver: *mut c_void) -> ipasir2_errorcode {
    ipasir_release(solver);

    ipasir2_errorcode::IPASIR2_E_OK
}

/// Returns the supported configuration options.
/// # Safety
/// Recovers a context bundle from a raw pointer.
#[no_mangle]
pub unsafe extern "C" fn ipasir2_options(
    solver: *mut c_void,
    options: *const *mut ipasir2_option,
    count: *mut c_int,
) -> ipasir2_errorcode {
    ipasir2_errorcode::IPASIR2_E_UNSUPPORTED
}

/// Returns the handle to the option with the gien name.
/// # Safety
/// Recovers a context bundle from a raw pointer.
#[no_mangle]
pub unsafe extern "C" fn ipasir2_get_option_handle(
    solver: *mut c_void,
    name: *const c_char,
    handle: *const ipasir2_option,
) -> ipasir2_errorcode {
    ipasir2_errorcode::IPASIR2_E_UNSUPPORTED
}

/// Sets the value of the given option.
/// # Safety
/// Recovers a context bundle from a raw pointer.
#[no_mangle]
pub unsafe extern "C" fn ipasir2_set_option(
    solver: *mut c_void,
    handle: *const ipasir2_option,
    value: i64,
    index: i64,
) -> ipasir2_errorcode {
    ipasir2_errorcode::IPASIR2_E_UNSUPPORTED
}

/// Adds a clause to the solver.
///
/// The `proofmeta` structure is not supported.
/// # Safety
/// Recovers a context bundle and takes a clause from raw pointers.
#[allow(unused_variables)]
#[no_mangle]
pub unsafe extern "C" fn ipasir2_add(
    solver: *mut c_void,
    clause: *const i32,
    len: i32,
    forgettable: i32,
    proofmeta: *mut c_void,
) -> ipasir2_errorcode {
    if !proofmeta.is_null() {
        return ipasir2_errorcode::IPASIR2_E_UNSUPPORTED_ARGUMENT;
    }

    let clause = std::slice::from_raw_parts(clause, len as usize);

    let bundle: &mut ContextBundle = &mut *(solver as *mut ContextBundle);
    assert!(bundle.clause_buffer.is_empty());

    for literal in clause {
        let literal_atom = literal.unsigned_abs();
        bundle.context.ensure_atom(literal_atom);
        bundle
            .clause_buffer
            .push(CLiteral::new(literal_atom, literal.is_positive()));
    }

    bundle
        .context
        .add_clause_unchecked(std::mem::take(&mut bundle.clause_buffer));

    ipasir2_errorcode::IPASIR2_E_OK
}

/// Assumes the given literals and then calls solve on the context.
///
/// # Safety
/// Recovers a context bundle from a raw pointer.
#[no_mangle]
pub unsafe extern "C" fn ipasir2_solve(
    solver: *mut c_void,
    result: *mut c_int,
    literals: *const i32,
    len: i32,
) -> ipasir2_errorcode {
    let bundle: &mut ContextBundle = &mut *(solver as *mut ContextBundle);
    if len != 0 {
        let assumption_literals = std::slice::from_raw_parts(literals, len as usize);
        for assumption in assumption_literals {
            let literal_atom = assumption.unsigned_abs();
            bundle.context.ensure_atom(literal_atom);
            let assumption = CLiteral::new(literal_atom, assumption.is_positive());
            bundle.context.add_assumption(assumption);
        }
    }

    std::ptr::write(result, ipasir_solve(solver));

    ipasir2_errorcode::IPASIR2_E_OK
}

/// Returns the literal representing the value of the atom of the given literal, if a satisfying valuation has been found.
///
/// That is, given a literal of the form ±a, the function returns:
/// * +a, if a is bound to true on the satisfying valuation.
/// * -a, if a is bound to false on the satisfying valuation.
///
/// # Safety
/// Recovers a context bundle from a raw pointer.
#[no_mangle]
pub unsafe extern "C" fn ipasir2_value(
    solver: *mut c_void,
    lit: i32,
    result: *mut i32,
) -> ipasir2_errorcode {
    std::ptr::write(result, ipasir_val(solver, lit));

    ipasir2_errorcode::IPASIR2_E_OK
}

/// Checks if the given assumption was used to prove the unsatisfiability in the previous solve.
/// # Safety
/// Recovers a context bundle and takes a clause from raw pointers.
#[no_mangle]
pub unsafe extern "C" fn ipasir2_failed(
    solver: *mut c_void,
    lit: i32,
    result: *mut c_int,
) -> ipasir2_errorcode {
    std::ptr::write(result, ipasir_failed(solver, lit));

    ipasir2_errorcode::IPASIR2_E_OK
}

/// Sets a callback function used to request termination of a solve.
///
/// For consistency and simplicity, this wraps [ipasir_set_terminate].
///
/// # Safety
/// Recovers a context bundle and takes a clause from raw pointers.
#[no_mangle]
pub unsafe extern "C" fn ipasir2_set_terminate(
    solver: *mut c_void,
    data: *mut c_void,
    callback: Option<extern "C" fn(data: *mut c_void) -> c_int>,
) -> ipasir2_errorcode {
    ipasir_set_terminate(solver, data, callback);

    ipasir2_errorcode::IPASIR2_E_OK
}

/// Sets a callback function for receiving addition clauses from the context.
/// # Safety
/// Recovers a context bundle and reads from multiple C pointers.
#[no_mangle]
pub unsafe extern "C" fn ipasir2_set_export(
    solver: *mut c_void,
    data: *mut c_void,
    max_length: c_int,
    callback: Option<
        extern "C" fn(data: *mut c_void, clause: *const i32, len: i32, proofmeta: *mut c_void),
    >,
) -> ipasir2_errorcode {
    let bundle: &mut ContextBundle = &mut *(solver as *mut ContextBundle);

    match &mut bundle.context.ipasir_callbacks {
        None => {
            let callbacks = IpasirCallbacks {
                delete_callback: callback,
                delete_data: data,
                ..Default::default()
            };

            bundle.context.ipasir_callbacks = Some(callbacks);
        }

        Some(callbacks) => {
            callbacks.delete_callback = callback;
            callbacks.delete_data = data;
        }
    }

    ipasir2_errorcode::IPASIR2_E_OK
}

/// Sets a callback function to be used when a clause is deleted.
///
/// # Safety
/// Recovers a context bundle and reads from multiple C pointers.
#[no_mangle]
pub unsafe extern "C" fn ipasir2_delete(
    solver: *mut c_void,
    data: *mut c_void,
    callback: Option<
        extern "C" fn(data: *mut c_void, clause: *const i32, len: i32, proofmeta: *mut c_void),
    >,
) -> ipasir2_errorcode {
    let bundle: &mut ContextBundle = &mut *(solver as *mut ContextBundle);

    match &mut bundle.context.ipasir_callbacks {
        None => {
            let callbacks = IpasirCallbacks {
                export_callback: callback,
                addition_data: data,
                ..Default::default()
            };

            bundle.context.ipasir_callbacks = Some(callbacks);
        }

        Some(callbacks) => {
            callbacks.export_callback = callback;
            callbacks.addition_data = data;
        }
    }

    ipasir2_errorcode::IPASIR2_E_OK
}

/// Sets a callback function for importing a clause into the context.
///
/// For the moment, unsupported.
/// See comments for details.
///
/// # Safety
/// Recovers a context bundle and reads from multiple C pointers.
#[no_mangle]
pub unsafe extern "C" fn ipasir2_set_import(
    solver: *mut c_void,
    data: *mut c_void,
    callback: Option<extern "C" fn(data: *mut c_void)>,
) -> ipasir2_errorcode {
    /*
    TODO: ipasir2_set_import
    From the paper this should be called periodically during a solve (e.g. at the start of a loop, or during an interrupt).

    On calling, a single clause may be added to the solver through the ipasir2_add callback.
    If multiple clauses are to be added, repeated calls should be made.
    And, that no further clauses are to be added is indicated by no call to ipasir2_add.

    So, ipasir2_add should set some state, and after the callback solve should branch on the state.
    As this will introduce some complexity to the solve, it will be implemented when the API is finalised.
     */

    ipasir2_errorcode::IPASIR2_E_UNSUPPORTED
}

/// Sets a callback for notification of fixed assignments.
///
/// For the moment, unsupported.
///
/// # Safety
/// Recovers a context bundle and reads from multiple C pointers.
#[no_mangle]
pub unsafe extern "C" fn ipasir2_set_fixed(
    solver: *mut c_void,
    data: *mut c_void,
    callback: Option<extern "C" fn(data: *mut c_void, fixed: i32)>,
) -> ipasir2_errorcode {
    /*
    TODO: ipasir2_set_fixed
    At present the callback is not made for literals fixed relative to assumptions made.
     */

    let bundle: &mut ContextBundle = &mut *(solver as *mut ContextBundle);

    match &mut bundle.context.ipasir_callbacks {
        None => {
            let callbacks = IpasirCallbacks {
                fixed_callback: callback,
                fixed_data: data,
                ..Default::default()
            };

            bundle.context.ipasir_callbacks = Some(callbacks);
        }

        Some(callbacks) => {
            callbacks.fixed_callback = callback;
            callbacks.fixed_data = data;
        }
    }

    ipasir2_errorcode::IPASIR2_E_OK
}
