//! Template bindings for IPASIR API.
//!
//! For the moment partial.

use crate::{
    context::ContextState,
    dispatch::library::report::SolveReport,
    ipasir::{ContextBundle, IPASIR_SIGNATURE},
    structures::{
        atom::Atom,
        clause::cClause,
        literal::{cLiteral, Literal},
    },
};
use std::ffi::{c_char, c_int, c_void};

/// # Safety
/// Writes the signature a raw pointer.
#[no_mangle]
pub unsafe extern "C" fn ipasir_signature(signature: *mut *const c_char) {
    *signature = IPASIR_SIGNATURE.as_ptr();
}

/// Initialises a solver a binds the given pointer to it's address.
/// # Safety
/// Releases the initialised solver to a raw pointer.
#[no_mangle]
pub unsafe extern "C" fn ipasir_init(solver: *mut *mut c_void) {
    let the_bundle = ContextBundle::default();
    assert!(the_bundle.context.state.eq(&ContextState::Configuration));

    let boxed_context = Box::new(the_bundle);
    *solver = Box::into_raw(boxed_context) as *mut c_void;
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
/// The `proofmeta` structure is not supported.
/// # Safety
/// Recovers a context bundle and takes a clause from raw pointers.
#[allow(unused_variables)]
#[no_mangle]
pub unsafe extern "C" fn ipasir_add(solver: *mut c_void, lit_or_zero: c_int) {
    let bundle: &mut ContextBundle = &mut *(solver as *mut ContextBundle);

    match lit_or_zero {
        0 => {
            let clause = std::mem::take(&mut bundle.clause_buffer);
            bundle.context.add_clause(clause);
        }
        literal => {
            let literal_atom = literal.unsigned_abs();
            match bundle.ei_map.get(&literal_atom) {
                None => {
                    let Ok(fresh_atom) = bundle.context.fresh_atom() else {
                        std::process::exit(1);
                    };
                    assert!(bundle.ie_map.len().eq(&(fresh_atom as usize)));
                    bundle.ei_map.insert(literal_atom, fresh_atom);
                    bundle.ie_map.push(literal_atom);
                    bundle
                        .clause_buffer
                        .push(cLiteral::fresh(fresh_atom, literal.is_positive()));
                }
                Some(atom) => {
                    bundle
                        .clause_buffer
                        .push(cLiteral::fresh(*atom, literal.is_positive()));
                }
            }
        }
    }
}

/// Calls solve on the context bundle.
///
/// # Safety
/// Recovers a context bundle from a raw pointer.
#[no_mangle]
pub unsafe extern "C" fn ipasir_solve(solver: *mut c_void) -> c_int {
    let bundle: &mut ContextBundle = &mut *(solver as *mut ContextBundle);

    let solve_result = bundle.context.solve();

    // As this is incremental, prepare for another solve.
    // This clears the *current* valuation, but if a satisfying valuation was found, it will be preserved in the prior valuation.
    bundle.context.backjump(0);
    bundle.context.remove_assumptions();

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
///*-
/// # Safety
/// Recovers a context bundle from a raw pointer.
#[no_mangle]
pub unsafe extern "C" fn ipasir_val(solver: *mut c_void, lit: i32) -> i32 {
    let bundle: &mut ContextBundle = &mut *(solver as *mut ContextBundle);

    if bundle.context.state != ContextState::Satisfiable {
        return 0;
    }

    let internal_atom = match bundle.ei_map.get(&lit.unsigned_abs()) {
        Some(atom) => atom,
        None => return 0,
    };

    // Following the note on solve, this uses the previous value of the atom as valuations are cleared after a solve completes.
    match bundle.context.atom_db.previous_value_of(*internal_atom) {
        true => lit,
        false => -lit,
    }
}

#[no_mangle]
pub unsafe extern "C" fn ipasir_failed(solver: *mut c_void, lit: i32) -> c_int {
    todo!()
}

#[no_mangle]
pub unsafe extern "C" fn ipasir_set_terminate(
    solver: *mut c_void,
    data: *mut c_void,
    callback: Option<extern "C" fn(data: *mut c_void) -> c_int>,
) {
    todo!()
}
