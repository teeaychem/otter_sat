//! Template bindings for IPASIR API.
//!
//! For the moment partial.

use crate::{
    context::ContextState,
    dispatch::library::report::SolveReport,
    ipasir::{ContextBundle, IPASIR_SIGNATURE},
    structures::{
        atom::Atom,
        clause::{cClause, Clause},
        literal::{abLiteral, cLiteral, Literal},
    },
};
use std::ffi::{c_char, c_int, c_void};

use super::{IpasirClauseDBCallbacks, IpasirSolveCallbacks};

/// # Safety
/// Writes the signature a raw pointer.
#[no_mangle]
pub unsafe extern "C" fn ipasir_signature() -> *const c_char {
    IPASIR_SIGNATURE.as_ptr()
}

/// Initialises a solver a binds the given pointer to it's address.
/// # Safety
/// Releases the initialised solver to a raw pointer.
#[no_mangle]
pub unsafe extern "C" fn ipasir_init() -> *mut c_void {
    let the_bundle = ContextBundle::default();
    assert!(the_bundle.context.state.eq(&ContextState::Configuration));

    let boxed_context = Box::new(the_bundle);
    Box::into_raw(boxed_context) as *mut c_void
}

/// Releases bound solver.
/// # Safety
/// Recovers a context bundle from a raw pointer.
#[no_mangle]
pub unsafe extern "C" fn ipasir_release(solver: *mut c_void) {
    let bundle: &mut ContextBundle = &mut *(solver as *mut ContextBundle);

    Box::from_raw(bundle);
}

/// Adds a clause to the solver.
///
/// # Safety
/// Recovers a context bundle and takes a clause from raw pointers.
#[allow(unused_variables)]
#[no_mangle]
pub unsafe extern "C" fn ipasir_add(solver: *mut c_void, lit_or_zero: c_int) {
    let bundle: &mut ContextBundle = &mut *(solver as *mut ContextBundle);

    bundle.context.refresh();

    match lit_or_zero {
        0 => {
            let clause = std::mem::take(&mut bundle.clause_buffer);
            bundle.context.add_clause_unchecked(clause);
        }
        literal => {
            let literal_atom = literal.unsigned_abs();
            bundle.context.ensure_atom(literal_atom);
            bundle
                .clause_buffer
                .push(cLiteral::new(literal_atom, literal.is_positive()));
        }
    }
}

/// Adds an assumption to the solver.
///
/// # Safety
/// Recovers a context bundle and takes a clause from raw pointers.
#[no_mangle]
pub unsafe extern "C" fn ipasir_assume(solver: *mut c_void, lit: c_int) {
    let bundle: &mut ContextBundle = &mut *(solver as *mut ContextBundle);

    bundle.context.refresh();

    let literal_atom = lit.unsigned_abs();
    bundle.context.ensure_atom(literal_atom);

    let assumption = abLiteral::new(literal_atom, lit.is_positive());

    let result = bundle.context.add_assumption(assumption);
    // println!("Assuming: {assumption} \t Result: {result:?}");
}

/// Calls solve on the context bundle.
///
/// # Safety
/// Recovers a context bundle from a raw pointer.
#[no_mangle]
pub unsafe extern "C" fn ipasir_solve(solver: *mut c_void) -> c_int {
    let bundle: &mut ContextBundle = &mut *(solver as *mut ContextBundle);

    match bundle.context.refresh() {
        true => {
            bundle.core_keys.clear();
            bundle.core_literals.clear();
        }
        false => {}
    }

    let solve_result = bundle.context.solve();

    match solve_result {
        Ok(SolveReport::Satisfiable) => 10,
        Ok(SolveReport::Unsatisfiable) => 20,
        _ => 0,
    }
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
pub unsafe extern "C" fn ipasir_val(solver: *mut c_void, lit: i32) -> i32 {
    let bundle: &mut ContextBundle = &mut *(solver as *mut ContextBundle);

    if bundle.context.state != ContextState::Satisfiable {
        return 0;
    }

    let literal_atom = lit.unsigned_abs();

    match bundle.context.atom_db.value_of(literal_atom) {
        Some(true) => lit,
        Some(false) => -lit,
        None => panic!("!"),
    }
}

/// If the formula is unsatisfiable, returns whether a literal is present in the identified unsatisfiable core.
///
/// Note, this is a strict expansion of the IPASIR API requirement, which is undefined on any `lit` which is not an assumption.
///
/// Specifically:
/// - If the formula is unsatisfiable:
///   + Returns 1, if the given literal is present in the unsatisfiable core, and 0 otherwise.
///   + Returns 0, otherwise.
/// - Otherwise, returns -1.
///
/// # Safety
/// Recovers a context bundle from a raw pointer.
#[no_mangle]
pub unsafe extern "C" fn ipasir_failed(solver: *mut c_void, lit: i32) -> c_int {
    let bundle: &mut ContextBundle = &mut *(solver as *mut ContextBundle);
    let ContextState::Unsatisfiable(_) = bundle.context.state else {
        return -1;
    };

    if bundle.core_literals.is_empty() {
        bundle.core_keys = bundle.context.core_keys();
        for key in &bundle.core_keys {
            let clause = bundle.context.clause_db.get_unchecked(key).unwrap();
            for literal in clause.literals() {
                bundle.core_literals.insert(literal);
            }
        }
    }

    let literal_canonical = abLiteral::new(lit.unsigned_abs(), lit.is_positive());

    match bundle.core_literals.contains(&literal_canonical) {
        true => 1,
        false => 0,
    }
}

/// Sets a callback function used to request termination of a solve.
///
/// # Safety
/// Recovers a context bundle from a raw pointer.
#[no_mangle]
pub unsafe extern "C" fn ipasir_set_terminate(
    solver: *mut c_void,
    data: *mut c_void,
    callback: Option<extern "C" fn(data: *mut c_void) -> c_int>,
) {
    let bundle: &mut ContextBundle = &mut *(solver as *mut ContextBundle);

    match &mut bundle.context.ipasir_callbacks {
        None => {
            let callbacks = IpasirSolveCallbacks {
                ipasir_terminate_callback: callback,
                ipasir_terminate_data: data,
            };

            bundle.context.ipasir_callbacks = Some(callbacks);
        }
        Some(callbacks) => {
            callbacks.ipasir_terminate_callback = callback;
            callbacks.ipasir_terminate_data = data;
        }
    }
}

/// Sets a callback function used to extract addition clauses up to a given length.
///
/// # Safety
/// Recovers a context bundle from a raw pointer.
#[no_mangle]
pub unsafe extern "C" fn ipasir_set_learn(
    solver: *mut c_void,
    data: *mut c_void,
    max_length: c_int,
    learn: Option<extern "C" fn(data: *mut c_void, clause: *mut i32)>,
) {
    let bundle: &mut ContextBundle = &mut *(solver as *mut ContextBundle);

    match &mut bundle.context.clause_db.ipasir_callbacks {
        None => {
            let callbacks = IpasirClauseDBCallbacks {
                ipasir_addition_callback: learn,
                ipasir_addition_callback_length: max_length as u32,
                ipasir_addition_data: data,
            };

            bundle.context.clause_db.ipasir_callbacks = Some(callbacks);
        }
        Some(callbacks) => {
            callbacks.ipasir_addition_callback = learn;
            callbacks.ipasir_addition_callback_length = max_length as u32;
            callbacks.ipasir_addition_data = data;
        }
    }
}
