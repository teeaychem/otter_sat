/*!
Partial bindings for IPASIR2 API.

In particular:
- There is no support for options.\
  Support will be added when (or before) the API is finalised.
- There is no support for set_import.
- There is partial support for set_fixed.
- There is no support for noting that an added clause may be forgotten.

Otherwise, if an IPASIR2 function wraps an IPASIR function, initial support is implemented.
*/

use crate::{
    db::clause::db_clause::DBClause,
    ipasir::{
        ContextBundle,
        ipasir_one::{ipasir_failed, ipasir_init},
    },
    structures::{
        clause::{Clause, ClauseSource, IntClause},
        literal::{CLiteral, Literal},
    },
};

use super::ipasir_one::{
    ipasir_release, ipasir_set_terminate, ipasir_signature, ipasir_solve, ipasir_val,
};

use std::ffi::{c_char, c_int, c_void};

/// Codes used to indicate the success or failure of a function call.
#[repr(C)]
pub enum ipasir2_errorcode {
    /// The call succeeded.
    IPASIR2_E_OK = 0,

    /// The call failed for an unknown reason.
    IPASIR2_E_UNKNOWN = 1,

    /// The function is not implemented.
    IPASIR2_E_UNSUPPORTED,

    /// The function does not support the given argument value.
    IPASIR2_E_UNSUPPORTED_ARGUMENT,

    /// The function does not support the given option.
    IPASIR2_E_UNSUPPORTED_OPTION,

    /// The function is not permitted given the current state of the solver.
    IPASIR2_E_INVALID_STATE,

    /// The call failed due to an invalid argument.
    IPASIR2_E_INVALID_ARGUMENT,

    /// The option value is outside the allowed range.
    IPASIR2_E_INVALID_OPTION_VALUE,
}

/// States of the context, these are a subset of [ContextState].
#[repr(C)]
pub enum ipasir2_state {
    /// The context allows for configuration.
    IPASIR2_S_CONFIG = 0,

    /// The context allows input.
    IPASIR2_S_INPUT = 1,

    /// The database is known to be consistent, e.g. with a complete valuation.
    IPASIR2_S_SAT,

    /// The database is known to be inconsistnet, e.g. with an unsatisfiable clause identified.
    IPASIR2_S_UNSAT,

    /// The consistency of the database is unknown.
    IPASIR2_S_SOLVING,
}

/// IPASIR Configuration Options
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
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ipasir2_signature(signature: *mut *const c_char) -> ipasir2_errorcode {
    unsafe { std::ptr::write(signature, ipasir_signature()) };

    ipasir2_errorcode::IPASIR2_E_OK
}

/// Initialises a solver a binds the given pointer to its address.
/// # Safety
/// Releases the initialised solver to a raw pointer.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ipasir2_init(solver: *mut *mut c_void) -> ipasir2_errorcode {
    unsafe { std::ptr::write(solver, ipasir_init()) };

    ipasir2_errorcode::IPASIR2_E_OK
}

/// Releases the bound solver, so long as it is not solving.
/// # Safety
/// Recovers a context bundle from a raw pointer.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ipasir2_release(solver: *mut c_void) -> ipasir2_errorcode {
    unsafe { ipasir_release(solver) };

    ipasir2_errorcode::IPASIR2_E_OK
}

/// Returns the supported configuration options.
/// # Safety
/// Recovers a context bundle from a raw pointer.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ipasir2_options(
    solver: *mut c_void,
    options: *const *mut ipasir2_option,
    count: *mut c_int,
) -> ipasir2_errorcode {
    ipasir2_errorcode::IPASIR2_E_UNSUPPORTED
}

/// Returns the handle to the option with the given name.
/// # Safety
/// Recovers a context bundle from a raw pointer.
#[unsafe(no_mangle)]
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
#[unsafe(no_mangle)]
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
#[unsafe(no_mangle)]
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

    let clause = unsafe { std::slice::from_raw_parts(clause, len as usize) };

    let bundle: &mut ContextBundle = unsafe { &mut *(solver as *mut ContextBundle) };
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
        .add_clause(std::mem::take(&mut bundle.clause_buffer));

    ipasir2_errorcode::IPASIR2_E_OK
}

/// Assumes the given literals and then calls solve on the context.
///
/// # Safety
/// Recovers a context bundle from a raw pointer.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ipasir2_solve(
    solver: *mut c_void,
    result: *mut c_int,
    literals: *const i32,
    len: i32,
) -> ipasir2_errorcode {
    let bundle: &mut ContextBundle = unsafe { &mut *(solver as *mut ContextBundle) };
    if len != 0 {
        let assumption_literals = unsafe { std::slice::from_raw_parts(literals, len as usize) };
        for assumption in assumption_literals {
            let assumption = CLiteral::from(*assumption);
            bundle.assumptions.push(assumption);
        }
    }

    unsafe { std::ptr::write(result, ipasir_solve(solver)) };

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
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ipasir2_value(
    solver: *mut c_void,
    lit: i32,
    result: *mut i32,
) -> ipasir2_errorcode {
    unsafe { std::ptr::write(result, ipasir_val(solver, lit)) };

    ipasir2_errorcode::IPASIR2_E_OK
}

/// Checks if the given assumption was used to prove the unsatisfiability in the previous solve.
/// # Safety
/// Recovers a context bundle and takes a clause from raw pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ipasir2_failed(
    solver: *mut c_void,
    lit: i32,
    result: *mut c_int,
) -> ipasir2_errorcode {
    unsafe { std::ptr::write(result, ipasir_failed(solver, lit)) };

    ipasir2_errorcode::IPASIR2_E_OK
}

/// Sets a callback function used to request termination of a solve.
///
/// For consistency and simplicity, this wraps [ipasir_set_terminate].
///
/// # Safety
/// Recovers a context bundle and takes a clause from raw pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ipasir2_set_terminate(
    solver: *mut c_void,
    data: *mut c_void,
    callback: Option<extern "C" fn(data: *mut c_void) -> c_int>,
) -> ipasir2_errorcode {
    unsafe { ipasir_set_terminate(solver, data, callback) };

    ipasir2_errorcode::IPASIR2_E_OK
}

/// Sets a callback function for receiving addition clauses from the context.
/// At present, no metadata is supported.
/// # Safety
/// Recovers a context bundle and reads from multiple C pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ipasir2_set_export(
    solver: *mut c_void,
    data: *mut c_void,
    max_length: c_int,
    callback: Option<
        extern "C" fn(data: *mut c_void, clause: *const i32, len: i32, proofmeta: *mut c_void),
    >,
) -> ipasir2_errorcode {
    if let Some(callback) = callback {
        let bundle: &mut ContextBundle = unsafe { &mut *(solver as *mut ContextBundle) };

        let callback = Box::new(move |clause: &DBClause, _: &ClauseSource| {
            if clause.len() < (max_length as usize) {
                let mut int_clause: Vec<c_int> = clause.literals().map(|l| l.into()).collect();
                let callback_ptr: *mut i32 = int_clause.as_mut_ptr();

                callback(
                    data,
                    callback_ptr,
                    clause.len() as i32,
                    std::ptr::null_mut(),
                );
            }
        });

        bundle.context.set_callback_addition(callback);

        ipasir2_errorcode::IPASIR2_E_OK
    } else {
        ipasir2_errorcode::IPASIR2_E_INVALID_ARGUMENT
    }
}

/// Sets a callback function to be used when a clause is deleted.
///
/// # Safety
/// Recovers a context bundle and reads from multiple C pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ipasir2_delete(
    solver: *mut c_void,
    data: *mut c_void,
    callback: Option<
        extern "C" fn(data: *mut c_void, clause: *const i32, len: i32, proofmeta: *mut c_void),
    >,
) -> ipasir2_errorcode {
    if let Some(callback) = callback {
        let bundle: &mut ContextBundle = unsafe { &mut *(solver as *mut ContextBundle) };

        let callback = Box::new(move |clause: &DBClause| {
            let callback_ptr: *mut i32 = if cfg!(feature = "boolean") {
                let mut int_clause: IntClause = clause.literals().map(|l| l.into()).collect();
                int_clause.as_mut_ptr()
            } else {
                clause.as_ptr() as *mut i32
            };

            match clause.size().try_into() {
                Ok(clause_size) => {
                    callback(data, callback_ptr, clause_size, std::ptr::null_mut());
                }
                Err(_) => {
                    log::error!("Clause too large for IPASIR delete callback");
                }
            }
        });

        bundle.context.set_callback_delete(callback);

        ipasir2_errorcode::IPASIR2_E_OK
    } else {
        ipasir2_errorcode::IPASIR2_E_INVALID_ARGUMENT
    }
}

/// Sets a callback function for importing a clause into the context.
///
/// For the moment, unsupported.
/// See comments for details.
///
/// # Safety
/// Recovers a context bundle and reads from multiple C pointers.
#[unsafe(no_mangle)]
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
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ipasir2_set_fixed(
    solver: *mut c_void,
    data: *mut c_void,
    callback: Option<extern "C" fn(data: *mut c_void, fixed: i32)>,
) -> ipasir2_errorcode {
    /*
    TODO: ipasir2_set_fixed
    At present the callback is not made for literals fixed relative to assumptions made.
     */
    if let Some(callback) = callback {
        let bundle: &mut ContextBundle = unsafe { &mut *(solver as *mut ContextBundle) };

        let callback = Box::new(move |literal: CLiteral| {
            callback(data, literal);
        });

        bundle.context.set_callback_fixed(callback);

        ipasir2_errorcode::IPASIR2_E_OK
    } else {
        ipasir2_errorcode::IPASIR2_E_INVALID_ARGUMENT
    }
}
