use crate::structures::{clause::CClause, literal::CLiteral};

use super::ClauseDB;

pub type CallbackAddition = dyn FnMut(&CClause);
pub type CallbackDelete = dyn FnMut(&CClause);
pub type CallbackFixed = dyn FnMut(CLiteral);

impl ClauseDB {
    pub fn set_callback_addition(&mut self, callback: Box<CallbackAddition>) {
        self.callback_addition = Some(callback);
    }

    pub fn make_callback_addition(&mut self, clause: &CClause) {
        if let Some(callback) = &mut self.callback_addition {
            callback(clause);
        }
    }

    pub fn set_callback_fixed(&mut self, callback: Box<CallbackFixed>) {
        self.callback_fixed = Some(callback);
    }

    pub fn make_callback_fixed(&mut self, literal: CLiteral) {
        if let Some(callback) = &mut self.callback_fixed {
            callback(literal);
        }
    }

    pub fn set_callback_delete(&mut self, callback: Box<CallbackDelete>) {
        self.callback_delete = Some(callback);
    }

    pub fn make_callback_delete(&mut self, clause: &CClause) {
        if let Some(callback) = &mut self.callback_delete {
            callback(clause);
        }
    }
}
