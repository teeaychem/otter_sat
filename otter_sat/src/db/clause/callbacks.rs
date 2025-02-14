use crate::{
    context::callbacks::{CallbackOnClause, CallbackOnClauseSource, CallbackOnLiteral},
    structures::{clause::ClauseSource, literal::CLiteral},
};

use super::{db_clause::dbClause, ClauseDB};

impl ClauseDB {
    /// Set a callback to be made when an original clause is added to the context.
    pub fn set_callback_original(&mut self, callback: Box<CallbackOnClauseSource>) {
        self.callback_original = Some(callback);
    }

    /// Set a callback to be made when an addition clause is added to the context.
    pub fn set_callback_addition(&mut self, callback: Box<CallbackOnClauseSource>) {
        self.callback_addition = Some(callback);
    }

    /// Set a callback to be made when the value of a literal is fixed within a solve.
    pub fn set_callback_fixed(&mut self, callback: Box<CallbackOnLiteral>) {
        self.callback_fixed = Some(callback)
    }

    /// Set a callback to be made when a clause is deleted from the context.
    pub fn set_callback_delete(&mut self, callback: Box<CallbackOnClause>) {
        self.callback_delete = Some(callback);
    }

    /// Set a callback to be made when the context is identified as unsatisfiable.
    pub fn set_callback_unsatisfiable(&mut self, callback: Box<CallbackOnClause>) {
        self.callback_unsatisfiable = Some(callback);
    }
}

impl ClauseDB {
    /// Make the callback to be made when an original clause is added to the context.
    pub fn make_callback_original(&mut self, clause: &dbClause, source: &ClauseSource) {
        if let Some(callback) = &mut self.callback_original {
            callback(clause, source);
        }
    }

    /// Make the callback set to be made when an addition clause is added to the context.
    pub fn make_callback_addition(&mut self, clause: &dbClause, source: &ClauseSource) {
        if let Some(callback) = &mut self.callback_addition {
            callback(clause, source);
        }
    }

    /// Make the callback set to be made when the value of a literal is fixed within a solve.
    pub fn make_callback_fixed(&mut self, literal: CLiteral) {
        if let Some(callback) = &mut self.callback_fixed {
            callback(literal);
        }
    }

    /// Make the callback set to be made when a clause is deleted from the context.
    pub fn make_callback_delete(&mut self, clause: &dbClause) {
        if let Some(callback) = &mut self.callback_delete {
            callback(clause);
        }
    }

    /// Make the callback set to be made when the context is identified as unsatisfiable.
    pub fn make_callback_unsatisfiable(&mut self, clause: &dbClause) {
        if let Some(callback) = &mut self.callback_unsatisfiable {
            callback(clause);
        }
    }
}
