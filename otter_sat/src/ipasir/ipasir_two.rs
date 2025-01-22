use crate::{context::Context, ipasir::IPASIR_SIGNATURE};
use std::{ffi::c_void, os::raw};

#[no_mangle]
pub extern "C" fn test() {
    println!("test")
}

#[no_mangle]
pub extern "C" fn ipasir2_signature() -> *const raw::c_char {
    IPASIR_SIGNATURE.as_ptr()
}

#[no_mangle]
pub extern "C" fn ipasir2_init() -> *mut Context {
    let a_config = crate::config::Config::default();
    let the_context = Context::from_config(a_config, None);
    let boxed_context = Box::new(the_context);
    Box::into_raw(boxed_context)
}

#[no_mangle]
pub extern "C" fn ipasir2_release(solver: *mut Context) {
    unsafe {
        Box::from_raw(solver);
    }
}

#[allow(unused_variables)]
#[no_mangle]
pub extern "C" fn ipasir2_add(
    solver: *mut Context,
    clause: *const i32,
    len: i32,
    forgettable: i32,
    proofmeta: *mut c_void,
) {
    let clause = unsafe { std::slice::from_raw_parts(clause, len as usize) };
    println!("The clause : {clause:?}");
}
