//! Template bindings for IPASIR2 API.
//!
//! For the moment partial.

use crate::{
    context::ContextState,
    dispatch::library::report::SolveReport,
    ipasir::{ContextBundle, IPASIR_SIGNATURE},
    structures::{
        clause::CClause,
        literal::{CLiteral, Literal},
    },
};
use std::ffi::{c_char, c_int, c_void};

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

#[allow(non_camel_case_types)]
#[repr(C)]
pub enum ipasir2_state {
    IPASIR2_S_CONFIG = 0,
    IPASIR2_S_INPUT = 1,
    IPASIR2_S_SAT,
    IPASIR2_S_UNSAT,
    IPASIR2_S_SOLVING,
}

#[allow(non_camel_case_types)]
#[repr(C)]
pub struct ipasir2_option {
    name: *const c_char,
    min: i64,
    max: i64,
    max_state: ipasir2_state,
    tunable: c_int,
    indexed: c_int,
    handle: *const c_void,
}

/// # Safety
/// Writes the signature a raw pointer.
#[no_mangle]
pub unsafe extern "C" fn ipasir2_signature(signature: *mut *const c_char) -> ipasir2_errorcode {
    *signature = IPASIR_SIGNATURE.as_ptr();
    ipasir2_errorcode::IPASIR2_E_OK
}

/// Initialises a solver a binds the given pointer to it's address.
/// # Safety
/// Releases the initialised solver to a raw pointer.
#[no_mangle]
pub unsafe extern "C" fn ipasir2_init(solver: *mut *mut c_void) -> ipasir2_errorcode {
    let the_bundle = ContextBundle::default();
    assert!(the_bundle.context.state.eq(&ContextState::Configuration));

    let boxed_context = Box::new(the_bundle);
    *solver = Box::into_raw(boxed_context) as *mut c_void;

    ipasir2_errorcode::IPASIR2_E_OK
}

/// Releases bound solver, so long as it is not solving.
/// # Safety
/// Recovers a context bundle from a raw pointer.
#[no_mangle]
pub unsafe extern "C" fn ipasir2_release(solver: *mut c_void) -> ipasir2_errorcode {
    let bundle: &mut ContextBundle = &mut *(solver as *mut ContextBundle);

    if bundle.context.state == ContextState::Solving {
        return ipasir2_errorcode::IPASIR2_E_INVALID_STATE;
    }

    Box::from_raw(bundle);
    ipasir2_errorcode::IPASIR2_E_OK
}

#[no_mangle]
pub unsafe extern "C" fn ipasir2_options(
    solver: *mut c_void,
    options: *const *mut ipasir2_option,
    count: *mut c_int,
) -> ipasir2_errorcode {
    todo!()
}

#[no_mangle]
pub unsafe extern "C" fn ipasir2_get_option_handle(
    solver: *mut c_void,
    options: *const c_char,
    handle: *const ipasir2_option,
) -> ipasir2_errorcode {
    todo!()
}

#[no_mangle]
pub unsafe extern "C" fn ipasir2_set_option(
    solver: *mut c_void,
    handle: *const ipasir2_option,
    value: i64,
    index: i64,
) -> ipasir2_errorcode {
    todo!()
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

    let mut internal_clause: CClause = vec![];

    for literal in clause {
        let literal_atom = literal.unsigned_abs();
        bundle.context.ensure_atom(literal_atom);
        internal_clause.push(CLiteral::new(literal_atom, literal.is_positive()));
    }

    bundle.context.add_clause(internal_clause);

    ipasir2_errorcode::IPASIR2_E_OK
}

/// Calls solve on the context bundle.
///
///
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

    let solve_result = bundle.context.solve();

    // As this is incremental, prepare for another solve.
    // This clears the *current* valuation, but if a satisfying valuation was found, it will be preserved in the prior valuation.
    bundle.context.backjump(0);
    bundle.context.remove_assumptions();

    let result_code = match solve_result {
        Ok(SolveReport::Satisfiable) => 10,
        Ok(SolveReport::Unsatisfiable) => 20,
        _ => 0,
    };

    *result = result_code;

    ipasir2_errorcode::IPASIR2_E_OK
}

/// Returns the literal representing whether the value of the atom of the given literal, if a satisfying valuation has been found.
///
/// Explicitly, given a literal of the form ±a, `result` is set to:
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
    let bundle: &mut ContextBundle = &mut *(solver as *mut ContextBundle);

    if bundle.context.state != ContextState::Satisfiable {
        return ipasir2_errorcode::IPASIR2_E_INVALID_STATE;
    }

    // Following the note on solve, this uses the previous value of the atom as valuations are cleared after a solve completes.
    *result = match bundle.context.atom_db.previous_value_of(lit.unsigned_abs()) {
        true => lit,
        false => -lit,
    };

    ipasir2_errorcode::IPASIR2_E_OK
}

#[no_mangle]
pub unsafe extern "C" fn ipasir2_failed(
    solver: *mut c_void,
    lit: i32,
    result: *mut c_int,
) -> ipasir2_errorcode {
    todo!()
}

#[no_mangle]
pub unsafe extern "C" fn ipasir2_set_terminate(
    solver: *mut c_void,
    data: *mut c_void,
    callback: Option<extern "C" fn(data: *mut c_void) -> c_int>,
) -> ipasir2_errorcode {
    // TODO:

    ipasir2_errorcode::IPASIR2_E_UNSUPPORTED
}

#[no_mangle]
pub unsafe extern "C" fn ipasir2_set_export(
    solver: *mut c_void,
    data: *mut c_void,
    max_length: c_int,
    callback: Option<
        extern "C" fn(data: *mut c_void, clause: *const i32, len: i32, proofmeta: *mut c_void),
    >,
) -> ipasir2_errorcode {
    todo!()
}

#[no_mangle]
pub unsafe extern "C" fn ipasir2_delete(
    solver: *mut c_void,
    data: *mut c_void,
    callback: Option<
        extern "C" fn(data: *mut c_void, clause: *const i32, len: i32, proofmeta: *mut c_void),
    >,
) -> ipasir2_errorcode {
    todo!()
}

#[no_mangle]
pub unsafe extern "C" fn ipasir2_set_import(
    solver: *mut c_void,
    data: *mut c_void,
    callback: Option<extern "C" fn(data: *mut c_void)>,
) -> ipasir2_errorcode {
    todo!()
}

#[no_mangle]
pub unsafe extern "C" fn ipasir2_set_fixed(
    solver: *mut c_void,
    data: *mut c_void,
    callback: Option<extern "C" fn(data: *mut c_void, fixed: i32)>,
) -> ipasir2_errorcode {
    todo!()
}
