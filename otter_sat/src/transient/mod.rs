//! One-time structures.

use crate::{db::ClauseKey, structures::literal::CLiteral};
#[doc(hidden)]
mod cell;
pub mod config;
pub mod resolution_buffer;

/// Possilbe 'Ok' results from resolution using a resolution buffer.
pub enum ResolutionOk {
    /// A unique implication point was identified.
    UIP,

    /// Resolution produced a unit clause.
    UnitClause,

    /// Resolution identified a clause already in the database.
    Repeat(ClauseKey, CLiteral),
}
