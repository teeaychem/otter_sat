//! A struct holding a [clause](Clause) and associated metadata.
//!
//! For [clause trait](Clause) see [Clause], and for the canonical representation of a clause see [vClause].
//!
//! A [dbClause] contains:
//! - A [clause](Clause) (represented as a [vClause]).
//! - A [key](ClauseKey) used to access the [dbClause]/[clause](Clause).
//! - Other, useful, metadata.
//!
//! # Terminology
//! ## Watch candidate
//!   - A literal with an atom, on the current valuation, that either has no value, or is such that the value of the atom is the same as the polarity of the literal
//!
//! ## Watched literals
//!
//! Two distinguished watch candidates.
//!
//! In particular, watches are initialised for any clause containing two or more literals.
//!
//! At present, the literals watched are the *first* literal in the clause and the literal at the position of `watch_ptr`.
//! In order to preserve this invariant, order of literals in the claue is mutated as nedded.
//! For details on the way watched literals are updated, see implementations (notably [update_watch](dbClause::update_watch)).

use crate::{
    db::{atom::AtomDB, keys::ClauseKey},
    structures::{
        clause::{vClause, Clause},
        literal::abLiteral,
        valuation::vValuation,
    },
};

use std::ops::Deref;

#[doc(hidden)]
mod subsumption;
#[doc(hidden)]
mod watches;

/// A clause together with some metadata.
#[allow(non_camel_case_types)]
pub struct dbClause {
    /// A key for accessing the clause
    key: ClauseKey,
    /// The clause, stored instantiated as a [vClause].
    clause: vClause,
    /// Whether the clause is active.
    active: bool,
    /// The 'other' watched literal.
    watch_ptr: usize,
}

impl dbClause {
    /// Bundles a [ClauseKey] and [Clause] into a [dbClause] and initialises defaults.
    ///
    /// Note: This does not store the [dbClause] in the [clause database](crate::db::clause::ClauseDB).
    /// Instead, this is the canonical way to obtained some thing to be stored in a database.
    /// See, e.g. the [ClauseDB]((crate::db::clause::ClauseDB)) '[store](crate::db::clause::ClauseDB::store)' method for example use.
    pub fn from(
        key: ClauseKey,
        clause: vClause,
        atom_db: &mut AtomDB,
        valuation: Option<&vValuation>,
    ) -> Self {
        let mut db_clause = Self {
            key,
            clause,
            active: true,
            watch_ptr: 0,
        };

        db_clause.initialise_watches(atom_db, valuation);

        db_clause
    }

    /// The key used to access the [dbClause].
    pub const fn key(&self) -> ClauseKey {
        self.key
    }

    /// Whether the [dbClause] is active.
    pub fn is_active(&self) -> bool {
        self.active
    }

    /// Activates the [dbClause].
    pub fn activate(&mut self) {
        self.active = true
    }

    /// Deactivates the [dbClause].
    pub fn deactivate(&mut self) {
        self.active = false
    }
}

// Subsumption

impl std::fmt::Display for dbClause {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.clause.as_string())
    }
}

impl Deref for dbClause {
    type Target = [abLiteral];

    fn deref(&self) -> &Self::Target {
        &self.clause
    }
}
