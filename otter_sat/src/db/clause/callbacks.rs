use crate::{
    context::callbacks::{CallbackOnClause, CallbackOnClauseSource, CallbackOnLiteral},
    structures::{clause::ClauseSource, literal::CLiteral},
};

use super::{db_clause::dbClause, ClauseDB};

impl ClauseDB {
    pub fn set_callback_original(&mut self, callback: Box<CallbackOnClauseSource>) {
        self.callback_original = Some(callback);
    }

    pub fn set_callback_addition(&mut self, callback: Box<CallbackOnClauseSource>) {
        self.callback_addition = Some(callback);
    }

    pub fn set_callback_fixed(&mut self, callback: Box<CallbackOnLiteral>) {
        self.callback_fixed = Some(callback)
    }

    pub fn set_callback_delete(&mut self, callback: Box<CallbackOnClause>) {
        self.callback_delete = Some(callback);
    }

    pub fn set_callback_unsatisfiable(&mut self, callback: Box<CallbackOnClause>) {
        self.callback_unsatisfiable = Some(callback);
    }
}

impl ClauseDB {
    pub fn make_callback_original(&mut self, clause: &dbClause, source: &ClauseSource) {
        if let Some(callback) = &mut self.callback_original {
            callback(clause, source);
        }
    }

    pub fn make_callback_addition(&mut self, clause: &dbClause, source: &ClauseSource) {
        if let Some(callback) = &mut self.callback_addition {
            callback(clause, source);
        }
    }

    pub fn make_callback_fixed(&mut self, literal: CLiteral) {
        if let Some(callback) = &mut self.callback_fixed {
            callback(literal);
        }
    }

    pub fn make_callback_delete(&mut self, clause: &dbClause) {
        if let Some(callback) = &mut self.callback_delete {
            callback(clause);
        }
    }

    pub fn make_callback_unsatisfiable(&mut self, clause: &dbClause) {
        if let Some(callback) = &mut self.callback_unsatisfiable {
            callback(clause);
        }
    }
}
