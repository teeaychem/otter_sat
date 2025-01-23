use crate::{
    config::Config,
    context::{Context, ContextState},
    ipasir::IPASIR_SIGNATURE,
    structures::{
        atom::Atom,
        clause::{vClause, Clause},
        literal::{abLiteral, Literal},
    },
};
use std::{
    collections::HashMap,
    ffi::{c_char, c_int, c_void},
};

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
}

impl Default for ContextBundle {
    fn default() -> Self {
        ContextBundle {
            context: Context::from_config(Config::default(), None),
            ei_map: HashMap::default(),
            ie_map: vec![0],
        }
    }
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

    let mut internal_clause: vClause = vec![];

    for literal in clause {
        let literal_atom = literal.unsigned_abs();

        match bundle.ei_map.get(&literal_atom) {
            None => {
                let Ok(fresh_atom) = bundle.context.fresh_atom() else {
                    return ipasir2_errorcode::IPASIR2_E_UNKNOWN;
                };
                assert!(bundle.ie_map.len().eq(&(fresh_atom as usize)));
                bundle.ei_map.insert(literal_atom, fresh_atom);
                bundle.ie_map.push(literal_atom);
                internal_clause.push(abLiteral::fresh(fresh_atom, literal.is_positive()));
            }
            Some(atom) => {
                internal_clause.push(abLiteral::fresh(*atom, literal.is_positive()));
            }
        }
    }

    bundle.context.add_clause(internal_clause);

    ipasir2_errorcode::IPASIR2_E_OK
}

/// # Safety
/// Recovers a context bundle from a raw pointer.
#[no_mangle]
pub unsafe extern "C" fn ipasir2_solve(
    solver: *mut c_void,
    result: *mut c_int,
    literals: *const i32,
    len: i32,
) {
    let bundle: &mut ContextBundle = &mut *(solver as *mut ContextBundle);

    let result = bundle.context.solve();
    println!("{result:?}");
    println!("{}", bundle.context.atom_db.valuation_string());
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

    let internal_atom = match bundle.ei_map.get(&lit.unsigned_abs()) {
        Some(atom) => atom,
        None => return ipasir2_errorcode::IPASIR2_E_INVALID_ARGUMENT,
    };

    // let result = result.as_mut().unwrap();

    *result = match bundle.context.atom_db.value_of(*internal_atom) {
        Some(true) => lit,
        Some(false) => -lit,
        None => panic!("Hek"),
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
) -> ipasir2_errorcode {
    todo!()
}

#[no_mangle]
pub unsafe extern "C" fn ipasir2_set_export(
    solver: *mut c_void,
    data: *mut c_void,
    max_length: c_int,
) -> ipasir2_errorcode {
    todo!()
}

#[no_mangle]
pub unsafe extern "C" fn ipasir2_delete(
    solver: *mut c_void,
    data: *mut c_void,
    callback: extern "C" fn(
        data: *mut c_void,
        clause: *const i32,
        len: i32,
        proofmeta: *mut c_void,
    ),
) -> ipasir2_errorcode {
    todo!()
}

#[no_mangle]
pub unsafe extern "C" fn ipasir2_set_import(
    solver: *mut c_void,
    data: *mut c_void,
    callback: extern "C" fn(data: *mut c_void),
) -> ipasir2_errorcode {
    todo!()
}

#[no_mangle]
pub unsafe extern "C" fn ipasir2_set_fixed(
    solver: *mut c_void,
    data: *mut c_void,
    callback: extern "C" fn(data: *mut c_void, fixed: i32),
) -> ipasir2_errorcode {
    todo!()
}
