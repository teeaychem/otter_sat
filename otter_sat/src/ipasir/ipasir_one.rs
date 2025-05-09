/*!
Bindings for the IPASIR API.

For a general overview, see the [ipasir module](crate::ipasir).
*/

use crate::{
    context::ContextState,
    db::clause::db_clause::DBClause,
    ipasir::{ContextBundle, IPASIR_SIGNATURE},
    reports::Report,
    structures::{
        clause::{Clause, ClauseSource},
        literal::{CLiteral, Literal},
    },
};
use std::ffi::{c_char, c_int, c_void};

/// Returns the name and the version of this library.
///
/// # Safety
/// Writes the signature a raw pointer.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ipasir_signature() -> *const c_char {
    IPASIR_SIGNATURE
        .get_or_init(|| {
            std::ffi::CString::new(format!(
                "{} {}",
                env!("CARGO_PKG_NAME"),
                env!("CARGO_PKG_VERSION")
            ))
            .unwrap_or_default()
        })
        .as_ptr()
}

/// Initialises a context bundle and returns a pointer to it.
///
/// This pointer may then be used as a parameter in functions of the API.
///
/// After initialisation the context is in configuration state, which is functionally equivalent to being in input state, from the perspective of the API.
///
/// # Safety
/// Returns a raw pointer to the initialised context.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ipasir_init() -> *mut c_void {
    let ctx_bundle = ContextBundle::default();
    assert!(ctx_bundle.context.state.eq(&ContextState::Configuration));

    let boxed_context = Box::new(ctx_bundle);
    Box::into_raw(boxed_context) as *mut c_void
}

/// Releases the pointed instance of the context (and supporting structures).
///
/// # Safety
/// Recovers a context bundle from a raw pointer.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ipasir_release(solver: *mut c_void) {
    unsafe {
        let bundle: &mut ContextBundle = &mut *(solver as *mut ContextBundle);

        Box::from_raw(bundle);
    }
}

/// Adds a literal to, or finalises a, clause under construction.
///
/// Literals are non-zero [i32]s, with 0 used to indicate the termination of a clause.
///
/// A clause is added to a context when, and only when, it is finalised.
///
/// # Safety
/// Recovers a context bundle and takes a clause from raw pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ipasir_add(solver: *mut c_void, lit_or_zero: c_int) {
    let bundle: &mut ContextBundle = unsafe { &mut *(solver as *mut ContextBundle) };

    bundle.keep_fresh();

    match lit_or_zero {
        0 => {
            let clause = std::mem::take(&mut bundle.clause_buffer);
            bundle.context.add_clause(clause);
        }
        literal => {
            let literal_atom = literal.unsigned_abs();
            bundle.context.ensure_atom(literal_atom);
            bundle
                .clause_buffer
                .push(CLiteral::new(literal_atom, literal.is_positive()));
        }
    }
}

/// Adds an assumption to the context for the next solve.
///
/// # Safety
/// Recovers a context bundle from a raw pointer.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ipasir_assume(solver: *mut c_void, lit: c_int) {
    let bundle: &mut ContextBundle = unsafe { &mut *(solver as *mut ContextBundle) };

    bundle.keep_fresh();

    let lit = CLiteral::from(lit);

    bundle.assumptions.push(lit);
}

/// Calls solve on the given context and returns an integer detailing to the result of the solve.
///
/// The mapping used is:
/// - 10, if the formula was satisfiable.
/// - 20, if the formulas was unsatisfiable.
/// - 0, otherwise.
///
/// # Safety
/// Recovers a context bundle from a raw pointer.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ipasir_solve(solver: *mut c_void) -> c_int {
    let bundle: &mut ContextBundle = unsafe { &mut *(solver as *mut ContextBundle) };

    let assumptions = std::mem::take(&mut bundle.assumptions);
    let solve_result = bundle.context.solve_given(Some(assumptions));

    match solve_result {
        Ok(Report::Satisfiable) => 10,
        Ok(Report::Unsatisfiable) => 20,
        Ok(Report::Unknown) => {
            bundle.keep_fresh();
            0
        }
        Err(e) => panic!("{e:?}"),
    }
}

/// Returns the literal representing the value of the atom of the given literal.
///
/// Though, only if a satisfying valuation has been found.
///
/// That is, given a literal of the form ±a, the function returns:
/// * +a, if a is bound to true on the satisfying valuation.
/// * -a, if a is bound to false on the satisfying valuation.
///
/// # Safety
/// Recovers a context bundle from a raw pointer.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ipasir_val(solver: *mut c_void, lit: i32) -> i32 {
    let bundle: &mut ContextBundle = unsafe { &mut *(solver as *mut ContextBundle) };

    if bundle.context.state != ContextState::Satisfiable {
        return 0;
    }

    let literal_atom = lit.unsigned_abs();

    match bundle.context.value_of(literal_atom) {
        Some(true) => lit,
        Some(false) => -lit,
        None => panic!("! ipasir_val called with an incomplete valuation"),
    }
}

/// If the formula is unsatisfiable, returns whether a literal is present in the identified unsatisfiable core.
/// If so, the literal was used to prove the unsatisfiability of the formula.
///
/// Note, this is a strict expansion of the IPASIR API requirement, which is undefined on any `lit` which is not an assumption.
///
/// Specifically:
/// - If the formula is unsatisfiable:, the function returns:
///   + 1, if the given literal is present in the unsatisfiable core, and 0 otherwise.
///   + 0, otherwise.
/// - Otherwise, the function returns -1.
///
/// # Safety
/// Recovers a context bundle from a raw pointer.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ipasir_failed(solver: *mut c_void, lit: i32) -> c_int {
    let bundle: &mut ContextBundle = unsafe { &mut *(solver as *mut ContextBundle) };
    let ContextState::Unsatisfiable(_) = bundle.context.state else {
        return -1;
    };

    if bundle.failed_literals.is_empty() {
        bundle
            .failed_literals
            .extend(bundle.context.failed_assumpions());
    }

    let literal_canonical = CLiteral::from(lit);

    match bundle.failed_literals.contains(&literal_canonical.negate()) {
        true => 1,
        false => 0,
    }
}

/// Sets a callback function used to request termination of a solve.
///
/// The IPASIR API requires only that the callback is called 'periodically'.
/// As implemented, a call back is made at the start of every solve.
///
/// # Safety
/// Recovers a context bundle from a raw pointer.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ipasir_set_terminate(
    solver: *mut c_void,
    data: *mut c_void,
    callback: Option<extern "C" fn(data: *mut c_void) -> c_int>,
) {
    if let Some(callback) = callback {
        let bundle: &mut ContextBundle = unsafe { &mut *(solver as *mut ContextBundle) };

        let callback = Box::new(move || !matches!(callback(data), 0));
        bundle.context.set_callback_terminate_solve(callback);
    }
}

/// Sets a callback function used to extract addition (learnt) clauses up to the given length.
///
/// # Safety
/// Recovers a context bundle from a raw pointer.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ipasir_set_learn(
    solver: *mut c_void,
    data: *mut c_void,
    max_length: c_int,
    learn: Option<extern "C" fn(data: *mut c_void, clause: *mut i32)>,
) {
    if let Some(callback) = learn {
        let bundle: &mut ContextBundle = unsafe { &mut *(solver as *mut ContextBundle) };

        let callback_addition = Box::new(move |clause: &DBClause, _: &ClauseSource| {
            if clause.len() <= (max_length as usize) {
                let mut int_clause: Vec<c_int> = clause.literals().map(|l| l.into()).collect();
                int_clause.push(0);
                let callback_ptr: *mut i32 = int_clause.as_mut_ptr();
                callback(data, callback_ptr);
            }
        });

        bundle.context.set_callback_addition(callback_addition);

        let callback_original = Box::new(move |clause: &DBClause, _: &ClauseSource| {
            if clause.len() <= (max_length as usize) {
                let mut int_clause: Vec<c_int> = clause.literals().map(|l| l.into()).collect();
                int_clause.push(0);
                let callback_ptr: *mut i32 = int_clause.as_mut_ptr();
                callback(data, callback_ptr);
            }
        });

        bundle.context.set_callback_original(callback_original);
    }
}
