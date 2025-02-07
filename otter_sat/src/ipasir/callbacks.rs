//! A strcutre holding information for making IPASIR callbacks.
//!
//! # IPASIR Versions
//!
//! The structure is designed to support both IPASIR and IPASIR2 callbacks.
//!
//! In general, IPASIR and IPASIR2 callbacks are either unique or equivalent.
//! The exception is the callback for addition clauses, which includes additional parameters.
//! In this case, the addition method attempts the IPASIR callback, and only if this is not defined is the IPASIR2 callback attempted.
//!
//! # Implementation details
//!
//! The context stores an *optional* instance of the structure, allowing for a quick check of whether specific callbacks should be attempted.
//! And, if some structure is present in a context, methods associated with the structure are used.
//!
//!
use std::ffi::{c_int, c_void};

use crate::structures::{
    clause::{CClause, Clause, IntClause},
    literal::{CLiteral, IntLiteral, Literal},
};

/// Information regarding the solve callback.
pub struct IpasirCallbacks {
    /// Proofmeta, for IPASIR2 callbacks.
    pub proofmeta: *mut c_void,

    /// A holder for the IPASIR/2 terminate callback.
    pub terminate_callback: Option<extern "C" fn(data: *mut c_void) -> c_int>,

    pub terminate_data: *mut c_void,

    /// A holder for the IPASIR addition callback.
    pub learn_callback: Option<extern "C" fn(data: *mut c_void, clause: *mut i32)>,

    /// A holder for the IPASIR2 addition callback.
    pub export_callback: Option<
        extern "C" fn(data: *mut c_void, clause: *const i32, len: i32, proofmeta: *mut c_void),
    >,

    /// A holder for the maxumum clause length for application of the IPASIR/2 addition callback.
    pub addition_length: usize,

    /// A holder for data to be passed to applications of the IPASIR/2 addition callback.
    pub addition_data: *mut c_void,

    /// A holder for the IPASIR/2 delete callback.
    pub delete_callback: Option<
        extern "C" fn(data: *mut c_void, clause: *const i32, len: i32, proofmeta: *mut c_void),
    >,

    pub delete_data: *mut c_void,

    /// A holder for the IPASIR2 fixed callback.
    pub fixed_callback: Option<extern "C" fn(data: *mut c_void, fixed: i32)>,

    pub fixed_data: *mut c_void,
}

impl IpasirCallbacks {
    /// Calls the IPASIR/2 addition callback, if defined.
    ///
    /// # Safety
    /// Calls an external C function.
    #[allow(clippy::useless_conversion)]
    pub unsafe fn call_ipasir_addition_callback(&self, clause: &CClause) {
        if let Some(addition_callback) = self.learn_callback {
            if clause.size() <= self.addition_length {
                // The IPASIR callback requires a null terminated integer array.
                // So, a null terminated copy is made.

                let mut int_clause: Vec<c_int> = clause.literals().map(|l| l.into()).collect();
                int_clause.push(0);
                let callback_ptr: *mut i32 = int_clause.as_mut_ptr();

                addition_callback(self.addition_data, callback_ptr);
            }
        } else if let Some(addition_callback) = self.export_callback {
            if clause.size() <= self.addition_length {
                let callback_ptr: *mut i32 = if cfg!(feature = "boolean") {
                    let mut int_clause: IntClause = clause.literals().map(|l| l.into()).collect();
                    int_clause.as_mut_ptr()
                } else {
                    clause.as_ptr() as *mut i32
                };

                addition_callback(
                    self.addition_data,
                    callback_ptr,
                    clause.size().try_into().unwrap(),
                    self.proofmeta,
                );
            }
        }
    }

    /// Calls the IPASIR terminate callback, if defined.
    /// Otherwise, returns 0, simulating no termination request from a callback.
    /// # Safety
    /// Calls an external C function.
    pub unsafe fn call_ipasir_terminate_callback(&self) -> i32 {
        if let Some(terminate_callback) = self.terminate_callback {
            terminate_callback(self.terminate_data)
        } else {
            0
        }
    }

    /// Calls the IPASIR delete callback, if defined.
    /// # Safety
    /// Calls an external C function.
    #[allow(clippy::useless_conversion)]
    pub unsafe fn call_ipasir_delete_callback(&self, clause: &CClause) {
        if let Some(delete_callback) = self.delete_callback {
            let callback_ptr: *mut i32 = if cfg!(feature = "boolean") {
                let mut int_clause: IntClause = clause.literals().map(|l| l.into()).collect();
                int_clause.as_mut_ptr()
            } else {
                clause.as_ptr() as *mut i32
            };

            delete_callback(
                self.delete_data,
                callback_ptr,
                clause.size().try_into().unwrap(),
                self.proofmeta,
            );
        }
    }

    /// Calls the IPASIR2 fixed callback, if defined.
    /// # Safety
    /// Calls an external C function.
    #[allow(clippy::useless_conversion)]
    pub unsafe fn call_ipasir_fixed_callback(&self, literal: CLiteral) {
        if let Some(fixed_callback) = self.fixed_callback {
            #[cfg(feature = "boolean")]
            let literal: IntLiteral = literal.into();

            fixed_callback(self.fixed_data, literal)
        }
    }
}

impl Default for IpasirCallbacks {
    fn default() -> Self {
        Self {
            proofmeta: std::ptr::null_mut(),

            terminate_callback: None,
            terminate_data: std::ptr::null_mut(),

            learn_callback: None,
            export_callback: None,
            addition_length: 0,
            addition_data: std::ptr::null_mut(),

            delete_callback: None,
            delete_data: std::ptr::null_mut(),

            fixed_callback: None,
            fixed_data: std::ptr::null_mut(),
        }
    }
}
