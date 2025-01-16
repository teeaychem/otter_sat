//! Databases for holding information relevant to a solve.
//!
//!   - [The clause database](crate::db::clause)
//!     + A collection of clauses, each indexed by a clause key. \
//!       From an external perspective there are two important kinds of clause:
//!       * Original clauses \
//!         Original clauses are added to the context from some external source (e.g. directly or through some DIMACS file). \
//!         The collection of original clauses together with the collection of original literals are the CNF formula 𝐅 whose satisfiability may be determined.
//!       * Added clauses \
//!         Clauses added to the context by some procedure (e.g. via resolution).
//!         Every added clause is a consequence of the collection of original clauses.
//!
//!   - [The literal database](crate::db::literal)
//!     + The literal database handled structures who primary
//!       * The decision stack
//!   - [The atom database](crate::db::atom)
//!     + Properties of atoms.
//!       * Valuation
//!       * Watch database
//! - [Consequence queue](crate::db::consequence_q)

pub mod atom;
pub mod clause;
pub mod consequence_q;
mod keys;
pub use keys::*;
pub mod literal;

use std::borrow::Borrow;

use crate::{
    context::GenericContext,
    dispatch::{
        library::delta::{self, Delta},
        Dispatch,
    },
    structures::{
        clause::{Clause, Source as ClauseSource},
        literal::{abLiteral, Source as LiteralSource},
        valuation::vValuation,
    },
    types::err,
};

/// The index of a [decision level](crate::db::literal).
pub type LevelIndex = u32;

#[allow(non_camel_case_types)]
#[derive(PartialEq, Eq)]
/// The status of a database.
pub enum dbStatus {
    /// The database is known to be consistent, e.g. with a complete valuation.
    Consistent,
    /// The database is known to be inconsistnet, e.g. with an unsatisfiable clause identified.
    Inconsistent,
    /// The consistency of the database is unknown.
    Unknown,
}

impl std::fmt::Display for dbStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            dbStatus::Consistent => write!(f, "Consistent"),
            Self::Inconsistent => write!(f, "Inconsistent"),
            Self::Unknown => write!(f, "Unknown"),
        }
    }
}

/// Canonical methods to record literals and clauses to the context.
impl<R: rand::Rng + std::default::Default> GenericContext<R> {
    /// Records a literal in the appropriate database.
    ///
    /// ```rust, ignore
    /// self.record_literal(literal, literal::Source::BCP(key));
    /// ```
    ///
    /// If no decisions have been made, literals are added to the clause database as unit clauses.
    /// Otherwise, literals are recorded as consequences of the current decision.
    pub fn record_literal(&mut self, literal: impl Borrow<abLiteral>, source: LiteralSource) {
        match source {
            LiteralSource::PureLiteral => {
                //
                match self.literal_db.decision_count() {
                    0 => {
                        self.record_clause(*literal.borrow(), ClauseSource::PureLiteral, None);
                    }
                    _ => {
                        // Making a free decision is not supported after some other (non-free) decision has been made.
                        panic!("!")
                    }
                }
            }

            LiteralSource::BCP(_) => {
                //
                match self.literal_db.decision_count() {
                    0 => {
                        self.record_clause(*literal.borrow(), ClauseSource::BCP, None);
                    }
                    _ => unsafe {
                        self.literal_db
                            .top_mut_unchecked()
                            .record_consequence(literal, source)
                    },
                }
            }
        }
    }

    /// Records a clause and returns the key to the clause.
    /// If possible, a dispatch is sent with relevant details.
    ///
    /// ```rust, ignore
    /// let key = self.record_clause(resolved_clause, clause::Source::Resolution)?;
    ///
    /// let key = self.record_clause(literal, clause::Source::BCP)?;
    /// ```
    pub fn record_clause(
        &mut self,
        clause: impl Clause,
        source: ClauseSource,
        valuation: Option<&vValuation>,
    ) -> Result<ClauseKey, err::ClauseDB> {
        let key = self
            .clause_db
            .store(clause, source, &mut self.atom_db, valuation)?;

        if let Some(dispatcher) = &self.dispatcher {
            match key {
                ClauseKey::Unit(literal) => match source {
                    ClauseSource::PureLiteral => {
                        // TODO: Implement dispatches for free decisions
                    }

                    ClauseSource::BCP => {
                        let delta = delta::ClauseDB::BCP(ClauseKey::Unit(literal));
                        dispatcher(Dispatch::Delta(delta::Delta::ClauseDB(delta)));
                    }

                    ClauseSource::Resolution => {
                        let delta = delta::ClauseDB::Added(key);
                        dispatcher(Dispatch::Delta(delta::Delta::ClauseDB(delta)));
                    }

                    ClauseSource::Original => {
                        let delta = delta::ClauseDB::Original(key);
                        dispatcher(Dispatch::Delta(delta::Delta::ClauseDB(delta)));
                    }
                },

                _ => {
                    // Safety: The key was created above.
                    // TODO: Dispatches regarding literals could be made before the clause is stored to avoid the get…
                    let db_clause = unsafe { self.clause_db.get_unchecked(&key)? };
                    match db_clause.size() {
                        0 | 1 => panic!("!"),

                        _ => {
                            let delta = delta::ClauseDB::ClauseStart;
                            dispatcher(Dispatch::Delta(Delta::ClauseDB(delta)));
                            for literal in db_clause.literals() {
                                let delta = delta::ClauseDB::ClauseLiteral(*literal);
                                dispatcher(Dispatch::Delta(Delta::ClauseDB(delta)));
                            }

                            let delta = {
                                match source {
                                    ClauseSource::BCP | ClauseSource::PureLiteral => panic!("!"),
                                    ClauseSource::Original => delta::ClauseDB::Original(key),
                                    ClauseSource::Resolution => delta::ClauseDB::Added(key),
                                }
                            };
                            dispatcher(Dispatch::Delta(Delta::ClauseDB(delta)));
                        }
                    }
                }
            }
        }

        Ok(key)
    }
}
