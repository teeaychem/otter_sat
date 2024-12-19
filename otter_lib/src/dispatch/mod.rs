//! Dispatches for external observers.
//!
//! Dispatches have two uses:
//! - Communication after some procedure, e.g. a solve.
//! - Optional observation of the dynamics of a context and other related structures during some procedure.
//!
//! Each dispatch is a small message of some pre-determined type, and structures which may send dispatches optionally take a 'dispatcher' function to post dispatches.
//!
//! - [library] contains all dispatch types, arranged in a fixed heirarchy.
//! - [frat] contains tools for creating FRAT proofs by using dispatches.
//! - [core] contains tools for identifying unsatisfiable cores by using dispatches.
//!
//! Dispatches come in a variety of types;
//!
//! - [Deltas](crate::dispatch::library::delta), on some change during a procedure or to an interal structure.
//!   - For example:
//!     - Addition and deletion of clauses.
//!     - The use of a clause when applying resolution to a conflict.
//! - [Reports](crate::dispatch::library::report), on the result of some procedure.
//!   - For example, whether a formula is satisfiable.
//! - [Stats](crate::dispatch::library::stat), regarding various things.
//!   - For example, the number of expected and processed clauses when importing a DIMACS formla.
//!
//! Each type of dispatch has multiple subtypes.
//! If possible, these subtypes correspond to the procedure or structure from which the dispatch is sent.
//! For example, the `Delta` type of dispatch has deltas for boolean constraint propagation, the clause database, etc.
//!
//! Dispatches are designed to be tidy to deconstruct by pattern matching, though as a consequence are somewhat messy to construct.
//! Further, as the name of a type of dispatch may conflict with, e.g., the name of the structure the dispatch is related to, dispatch creation is designed to be made relative to the module of the type of dispatch.
//!
//! So, dispatches are typtically broken up into parts.
//! For example, the following dispatches information that a conflict was derived when applying bcp with `literal` freshly set due to the clause indexed by `clause_key`, with this dispatch itself being sent by passing it to the function `dispatcher`.
//!
//! ```ignore
//! let delta = delta::BCP::Conflict {
//!     from: literal,
//!     via: clause_key,
//! };
//! dispatcher(Dispatch::Delta(Delta::BCP(delta)));
//! ```
//!
//! After matching on the type `Delta` one may then match on `BCP`, and finally on `Conflict` to retreive this detail…
//!g
//! # Examples
//!
//! Dispatching details on the addition of an original clause to the clause database.
//!
//! The addition is communicated via a series of deltas, amounting to:
//! 1. A signal that a sequence of deltas relating to the addition of a clause are to follow.
//! 2. The details of some literal.
//! 3. The type of clause and the key with which the clause was stored.
//!
//! ```ignore
//! use crate::dispatch::library::delta::{self};
//!
//! if let Some(dispatcher) = &clause_db.dispatcher {
//!     let delta = delta::ClauseDB::ClauseStart;
//!     dispatcher(Dispatch::Delta(Delta::ClauseDB(delta)));
//!     for literal in &clause {
//!         let delta = delta::ClauseDB::ClauseLiteral(*literal);
//!         dispatcher(Dispatch::Delta(Delta::ClauseDB(delta)));
//!     }
//!     let delta = delta::ClauseDB::Original(the_key);
//!     dispatcher(Dispatch::Delta(Delta::ClauseDB(delta)));
//! }
//! ```
//!
//! Behaviour in line with these dispatches for a receiver may be to:
//! 1. Set up a buffer to record details of a clause.
//! 2. Record details of the literal to the buffer.
//! 3. Finalise a record of the buffer using the metadata of it's source in the context and the key used for internal access.
//!

use crate::context::Context;

pub mod core;
pub mod frat;
pub mod library;

/// Dispatch types.
#[derive(Clone)]
pub enum Dispatch {
    Delta(library::delta::Delta),
    Report(library::report::Report),
    Stat(library::stat::Stat),
}

impl Context {
    pub fn dispatch_active(&self) {
        if let Some(_d) = &self.dispatcher {
            self.clause_db.dispatch_active();
        }
    }
}

/// Ignores a dispatch, for external use.
pub fn hand(_: Dispatch) {}
