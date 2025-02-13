/*!
Callbacks associated with a context.

A context supports various callbacks, and
*/
use std::collections::HashSet;

use super::GenericContext;
use crate::{
    db::{clause::db_clause::dbClause, ClauseKey},
    structures::{clause::ClauseSource, literal::CLiteral},
};

pub type CallbackOnResolution = dyn FnMut(&HashSet<ClauseKey>, CLiteral);
pub type CallbackOnClauseSource = dyn FnMut(&dbClause, &ClauseSource);
pub type CallbackOnClause = dyn FnMut(&dbClause);
pub type CallbackOnLiteral = dyn FnMut(CLiteral);
pub type CallbackTerminate = dyn FnMut() -> bool;

/// Methods to set general callbacks.
impl<R: rand::Rng + std::default::Default> GenericContext<R> {
    pub fn set_callback_terminate(&mut self, callback: Box<CallbackTerminate>) {
        self.callback_terminate = Some(callback);
    }
}

/// Methods to set callbacks called within the clause database.
impl<R: rand::Rng + std::default::Default> GenericContext<R> {
    pub fn set_callback_original(&mut self, callback: Box<CallbackOnClauseSource>) {
        self.clause_db.set_callback_original(callback);
    }

    pub fn set_callback_addition(&mut self, callback: Box<CallbackOnClauseSource>) {
        self.clause_db.set_callback_addition(callback);
    }

    pub fn set_callback_fixed(&mut self, callback: Box<CallbackOnLiteral>) {
        self.clause_db.set_callback_fixed(callback);
    }

    pub fn set_callback_delete(&mut self, callback: Box<CallbackOnClause>) {
        self.clause_db.set_callback_delete(callback);
    }

    pub fn set_callback_unsatisfiable(&mut self, callback: Box<CallbackOnClause>) {
        self.clause_db.set_callback_unsatisfiable(callback);
    }
}

impl<R: rand::Rng + std::default::Default> GenericContext<R> {
    pub fn check_callback_terminate(&mut self) -> bool {
        if let Some(callback) = &mut self.callback_terminate {
            callback()
        } else {
            false
        }
    }
}
