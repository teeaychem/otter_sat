//! A struct holding a [clause](Clause) and associated metadata.
//!
//! For [clause trait](Clause) see [Clause], and for the canonical representation of a clause see [cClause].
//!
//! A [dbClause] contains:
//! - A [clause](Clause) (represented as a [cClause]).
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
        clause::{cClause, Clause},
        literal::cLiteral,
        valuation::vValuation,
    },
};

use std::{
    collections::HashSet,
    hash::{Hash, Hasher},
    ops::Deref,
};

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
    clause: cClause,

    /// Whether the clause is active.
    active: bool,

    /// The 'other' watched literal.
    watch_ptr: usize,

    /// Original clauses used to obtain the clause
    premises: HashSet<ClauseKey>,

    /// A count of the inferences the clause occurs in.
    inferences: usize,
}

impl dbClause {
    /// Bundles a [ClauseKey] and [Clause] into a [dbClause] and initialises non-watch defaults.
    ///
    /// Note:
    /// - This does not store the [dbClause] in the [clause database](crate::db::clause::ClauseDB).
    ///   Instead, this is the canonical way to obtain some thing to be stored in a database.
    pub fn new_unit(key: ClauseKey, literal: cLiteral, premises: HashSet<ClauseKey>) -> Self {
        Self {
            key,
            clause: vec![literal],
            active: true,
            watch_ptr: 0,
            premises,
            inferences: 0,
        }
    }

    /// Bundles a [ClauseKey] and [Clause] into a [dbClause] and initialises defaults.
    ///
    /// Note: This does not store the [dbClause] in the [clause database](crate::db::clause::ClauseDB).
    /// Instead, this is the canonical way to obtain some thing to be stored in a database.
    /// See, e.g. the [ClauseDB]((crate::db::clause::ClauseDB)) '[store](crate::db::clause::ClauseDB::store)' method for example use.
    ///
    /// A valuation is optional.
    /// If given, clauses are initialised with respect to the given valuation.
    /// Otherwise, clauses are initialised with respect to the current valuation of the context.
    pub fn new_nonunit(
        key: ClauseKey,
        clause: cClause,
        atom_db: &mut AtomDB,
        valuation: Option<&vValuation>,
        premises: HashSet<ClauseKey>,
    ) -> Self {
        let mut db_clause = dbClause {
            key,
            clause,
            active: true,
            watch_ptr: 0,
            premises,
            inferences: 0,
        };

        db_clause.initialise_watches(atom_db, valuation);

        db_clause
    }

    /// The key used to access the [dbClause].
    pub const fn key(&self) -> &ClauseKey {
        &self.key
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

    /// The clause stored.
    pub fn clause(&self) -> &cClause {
        &self.clause
    }

    pub fn premises(&self) -> &HashSet<ClauseKey> {
        &self.premises
    }

    pub fn proof_occurrence_count(&self) -> usize {
        self.inferences
    }

    pub fn increment_proof_count(&mut self) {
        self.inferences += 1
    }

    pub fn decrement_proof_count(&mut self) {
        self.inferences -= 1
    }
}

// Subsumption

impl std::fmt::Display for dbClause {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.clause.as_dimacs(false))
    }
}

impl Deref for dbClause {
    type Target = [cLiteral];

    fn deref(&self) -> &Self::Target {
        &self.clause
    }
}

impl PartialEq for dbClause {
    fn eq(&self, other: &Self) -> bool {
        self.key.eq(&other.key)
    }
}

impl Eq for dbClause {}

impl Hash for dbClause {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.key.hash(state);
    }
}
