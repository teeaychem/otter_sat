//! One-time structures.

use std::collections::HashSet;

use cell::Cell;
use config::BufferConfig;

use crate::{
    context::callbacks::CallbackOnPremises,
    db::ClauseKey,
    structures::{atom::Atom, literal::CLiteral},
};
#[doc(hidden)]
mod cell;

pub mod config;
#[doc(hidden)]
pub mod methods;

#[doc(hidden)]
pub mod valuation;

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
pub struct AtomCells {
    /// A count of literals in the clause whose atoms do not have a value on the given interpretation.
    valueless_count: usize,

    /// The length of the clause.
    clause_length: usize,

    /// The (direct) premises used top derive the clause.
    premises: HashSet<ClauseKey>,

    /// The buffer.
    buffer: Vec<Cell>,

    /// A stack of modified atoms, with the original value stored as literal polarity.
    merged_atoms: Vec<Atom>,

    /// A (typically derived) configuration for the instance of resolution.
    config: BufferConfig,

    /// The callback used on completion
    callback_premises: Option<Box<CallbackOnPremises>>,
}

impl AtomCells {
    /// Set the callback made when an instance of resolution completes and returns premises used to `callback`.
    pub fn set_callback_resolution_premises(&mut self, callback: Box<CallbackOnPremises>) {
        self.callback_premises = Some(callback);
    }

    /// Make the callback requested when an instance of resolution completes and returns premises used, if defined.
    pub fn make_callback_resolution_premises(&mut self, premises: &HashSet<ClauseKey>) {
        if let Some(callback) = &mut self.callback_premises {
            callback(premises);
        }
    }
}
