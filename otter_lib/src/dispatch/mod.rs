//! Dispatches from a context to external observers.
//!
//! - FRAT
//! - Unsatisfiable cores
//!
//! Unsat from FRAT, though not required.
//!
//! Examples.

use crate::context::Context;

pub mod core;
pub mod frat;
pub mod library;

#[derive(Clone)]
pub enum Dispatch {
    Delta(library::delta::Delta),
    Report(library::report::Report),
    Comment(library::comment::Comment),
    Stats(library::stat::Stat),
}

impl Context {
    pub fn dispatch_active(&self) {
        if let Some(d) = &self.dispatcher {
            self.clause_db.dispatch_active();

            for literal in self.literal_db.proven_literals() {
                let report = library::report::LiteralDB::Active(*literal);
                d(Dispatch::Report(library::report::Report::LiteralDB(report)));
            }
        }
    }
}

pub fn hand(_: Dispatch) {}
