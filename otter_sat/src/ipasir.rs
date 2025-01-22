use std::{ffi, os::raw};

use crate::context::Context;

const IPASIR_SIGNATURE: &std::ffi::CStr = c"otter_sat 0.0.10";

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
pub extern "C" fn ipasir2_release(S: *mut Context) {
    unsafe {
        Box::from_raw(S);
    }
}
