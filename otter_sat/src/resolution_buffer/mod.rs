//! One-time structures.

use std::collections::HashSet;

use cell::Cell;
use config::BufferConfig;

use crate::{db::ClauseKey, structures::literal::CLiteral};
#[doc(hidden)]
mod cell;
pub mod config;
#[doc(hidden)]
pub mod methods;

/// Possilbe 'Ok' results from resolution using a resolution buffer.
pub enum ResolutionOk {
    /// A unique implication point was identified.
    UIP,

    /// Resolution produced a unit clause.
    UnitClause,

    /// Resolution identified a clause already in the database.
    Repeat(ClauseKey, CLiteral),
}

/// A buffer for use when applying resolution to a sequence of clauses.
pub struct ResolutionBuffer {
    /// A count of literals in the clause whose atoms do not have a value on the given interpretation.
    valueless_count: usize,

    /// The length of the clause.
    clause_length: usize,

    /// The literal asserted by the current resolution candidate, if it exists..
    asserts: Option<CLiteral>,

    /// The (direct) premises used top derive the clause.
    premises: HashSet<ClauseKey>,

    /// The buffer.
    buffer: Vec<Cell>,

    /// A (typically derived) configuration for the instance of resolution.
    config: BufferConfig,

    /// The callback used on completion
    callback_premises: Option<Box<CallbackOnResolution>>,
}

pub type CallbackOnResolution = dyn FnMut(&HashSet<ClauseKey>);

impl ResolutionBuffer {
    pub fn set_callback_resolution_premises(&mut self, callback: Box<CallbackOnResolution>) {
        self.callback_premises = Some(callback);
    }

    pub fn make_callback_resolution_premises(&mut self, premises: &HashSet<ClauseKey>) {
        if let Some(callback) = &mut self.callback_premises {
            callback(premises);
        }
    }
}
