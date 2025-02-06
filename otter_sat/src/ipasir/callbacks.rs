use std::ffi::{c_int, c_void};

use crate::structures::{
    clause::{CClause, Clause, IntClause},
    literal::{CLiteral, IntLiteral, Literal},
};

/// Information regarding the solve callback.
pub struct IpasirCallbacks {
    pub proofmeta: *mut c_void,

    pub terminate_callback: Option<extern "C" fn(data: *mut c_void) -> c_int>,

    pub terminate_data: *mut c_void,

    /// IPASIR addition callback
    pub learn_callback: Option<extern "C" fn(data: *mut c_void, clause: *mut i32)>,

    /// IPASIR2 addition callback
    pub export_callback: Option<
        extern "C" fn(data: *mut c_void, clause: *const i32, len: i32, proofmeta: *mut c_void),
    >,

    pub addition_callback_length: u32,

    pub addition_data: *mut c_void,

    /// Delete callback
    pub delete_callback: Option<
        extern "C" fn(data: *mut c_void, clause: *const i32, len: i32, proofmeta: *mut c_void),
    >,

    pub delete_data: *mut c_void,

    pub fixed_callback: Option<extern "C" fn(data: *mut c_void, fixed: i32)>,

    pub fixed_data: *mut c_void,
}

impl IpasirCallbacks {
    /// Calls the IPASIR addition callback, if defined.
    ///
    /// # Safety
    /// The IPASIR API requires a pointer to the clause.
    /// And, transmute is used to transmute a const pointer to a mutable pointer, if integer literals are used.
    /// Safety can be restored by copying the clause, though this seems excessive.
    /// Though, regardless of this, the method calls an external C function.
    #[allow(clippy::useless_conversion)]
    pub unsafe fn call_ipasir_addition_callback(&self, clause: &CClause) {
        if let Some(addition_callback) = self.learn_callback {
            if clause.size() <= self.addition_callback_length as usize {
                if cfg!(feature = "boolean") {
                    let mut int_clause: IntClause = clause.literals().map(|l| l.into()).collect();
                    let callback_ptr: *mut i32 = int_clause.as_mut_ptr();

                    addition_callback(self.addition_data, callback_ptr);
                } else {
                    let clause_ptr: *const i32 = clause.as_ptr();
                    let callback_ptr: *mut i32 = clause_ptr as *mut i32;
                    addition_callback(self.addition_data, callback_ptr);
                };
            }
        }

        if let Some(addition_callback) = self.export_callback {
            if clause.size() <= self.addition_callback_length as usize {
                if cfg!(feature = "boolean") {
                    let mut int_clause: IntClause = clause.literals().map(|l| l.into()).collect();
                    let callback_ptr: *mut i32 = int_clause.as_mut_ptr();

                    addition_callback(
                        self.addition_data,
                        callback_ptr,
                        clause.size().try_into().unwrap(),
                        self.proofmeta,
                    );
                } else {
                    let clause_ptr: *const i32 = clause.as_ptr();
                    let callback_ptr: *mut i32 = clause_ptr as *mut i32;
                    addition_callback(
                        self.addition_data,
                        callback_ptr,
                        clause.size().try_into().unwrap(),
                        self.proofmeta,
                    );
                };
            }
        }
    }

    /// # Safety
    /// Calls an external C function.
    pub unsafe fn call_ipasir_terminate_callback(&self) -> i32 {
        if let Some(terminate_callback) = self.terminate_callback {
            terminate_callback(self.terminate_data)
        } else {
            1
        }
    }

    /// # Safety
    /// Calls an external C function.
    #[allow(clippy::useless_conversion)]
    pub unsafe fn call_ipasir_delete_callback(&self, clause: &CClause) {
        if let Some(delete_callback) = self.delete_callback {
            if cfg!(feature = "boolean") {
                let mut int_clause: IntClause = clause.literals().map(|l| l.into()).collect();
                let callback_ptr: *mut i32 = int_clause.as_mut_ptr();

                delete_callback(
                    self.delete_data,
                    callback_ptr,
                    clause.size().try_into().unwrap(),
                    self.proofmeta,
                );
            } else {
                let clause_ptr: *const i32 = clause.as_ptr();
                let callback_ptr: *mut i32 = clause_ptr as *mut i32;
                delete_callback(
                    self.delete_data,
                    callback_ptr,
                    clause.size().try_into().unwrap(),
                    self.proofmeta,
                );
            }
        }
    }

    /// # Safety
    /// Calls an external C function.
    #[allow(clippy::useless_conversion)]
    pub unsafe fn call_ipasir_fixed_callback(&self, literal: CLiteral) {
        if let Some(fixed_callback) = self.fixed_callback {
            if cfg!(feature = "boolean") {
                let int_literal: IntLiteral = literal.into();

                fixed_callback(self.fixed_data, int_literal);
            } else {
                fixed_callback(self.fixed_data, literal)
            }
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
            addition_callback_length: 0,
            addition_data: std::ptr::null_mut(),

            delete_callback: None,
            delete_data: std::ptr::null_mut(),

            fixed_callback: None,
            fixed_data: std::ptr::null_mut(),
        }
    }
}
