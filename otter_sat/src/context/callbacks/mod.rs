/*!
Callbacks associated with a context.

A context supports various callbacks, and
*/
use std::collections::HashSet;

use super::GenericContext;
use crate::{
    db::{ClauseKey, clause::db_clause::dbClause},
    structures::{clause::ClauseSource, literal::CLiteral},
};

/// The type of callback made on premises used during an instance of resolution.
pub type CallbackOnPremises = dyn FnMut(&HashSet<ClauseKey>);

/// The type of callback made on a clause and source of that clause.
pub type CallbackOnClauseSource = dyn FnMut(&dbClause, &ClauseSource);

/// The type of callback made on a clause.
pub type CallbackOnClause = dyn FnMut(&dbClause);

/// The type of callback made on a literal.
pub type CallbackOnLiteral = dyn FnMut(CLiteral);

/// The type of callback used to request termination of some procedure.
pub type CallbackTerminate = dyn FnMut() -> bool;

/// Methods to set general callbacks.
impl<R: rand::Rng + std::default::Default> GenericContext<R> {}

/// Methods to set callbacks called within the clause database.
impl<R: rand::Rng + std::default::Default> GenericContext<R> {
    /// Set a callback to be made when an original clause is added to the context.
    pub fn set_callback_original(&mut self, callback: Box<CallbackOnClauseSource>) {
        self.clause_db.set_callback_original(callback);
    }

    /// Set a callback to be made when an addition clause is added to the context.
    pub fn set_callback_addition(&mut self, callback: Box<CallbackOnClauseSource>) {
        self.clause_db.set_callback_addition(callback);
    }

    /// Set a callback to be made when the value of a literal is fixed within a solve.
    pub fn set_callback_fixed(&mut self, callback: Box<CallbackOnLiteral>) {
        self.clause_db.set_callback_fixed(callback);
    }

    /// Set a callback to be made when a clause is deleted from the context.
    pub fn set_callback_delete(&mut self, callback: Box<CallbackOnClause>) {
        self.clause_db.set_callback_delete(callback);
    }

    /// Set a callback to be made when the context is identified as unsatisfiable.
    pub fn set_callback_unsatisfiable(&mut self, callback: Box<CallbackOnClause>) {
        self.clause_db.set_callback_unsatisfiable(callback);
    }

    /// Set a callback to terminate a solve.
    pub fn set_callback_terminate_solve(&mut self, callback: Box<CallbackTerminate>) {
        self.callback_terminate = Some(callback);
    }

    /// Check whether the terminate callback has requested termination of a solve.
    pub fn check_callback_terminate_solve(&mut self) -> bool {
        if let Some(callback) = &mut self.callback_terminate {
            callback()
        } else {
            false
        }
    }
}
