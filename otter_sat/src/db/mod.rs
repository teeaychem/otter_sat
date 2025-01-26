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

use std::{borrow::Borrow, collections::HashSet};

use crate::{
    context::GenericContext,
    dispatch::{
        library::delta::{self, Delta},
        Dispatch,
    },
    structures::{
        clause::{Clause, Source as ClauseSource},
        consequence::{Consequence, Source as ConsequenceSource},
        literal::Literal,
        valuation::vValuation,
    },
    types::err::{self, ErrorKind},
};

/// The index of a [decision level](crate::db::literal).
pub type DecisionLevelIndex = u32;

/// Canonical methods to record literals and clauses to the context.
impl<R: rand::Rng + std::default::Default> GenericContext<R> {
    /// Records a literal in the appropriate database.
    ///
    /// ```rust, ignore
    /// let consequence = Consequence::from(literal, literal::Source::BCP(key));
    /// self.record_consequence(consequence);
    /// ```
    ///
    /// If no decisions have been made, literals are added to the clause database as unit clauses.
    /// Otherwise, literals are recorded as consequences of the current decision.
    ///
    /// # Origins
    /// If a propagation occurs without any decision having been made, then the valuation must conflict with each other literal in the clause.
    /// So, the origin of the unit is the origin of each literal.
    pub fn record_consequence(
        &mut self,
        consequence: impl Borrow<Consequence>,
    ) -> Result<(), ErrorKind> {
        let consequence = consequence.borrow().clone();
        match consequence.source() {
            ConsequenceSource::PureLiteral => {
                let origins = HashSet::default();
                // Making a free decision is not supported after some other (non-free) decision has been made.
                if !self.literal_db.is_decision_made() && self.literal_db.decision_count() == 0 {
                    self.record_clause(
                        *consequence.literal(),
                        ClauseSource::PureUnit,
                        None,
                        origins,
                    );
                } else {
                    panic!("! Origins")
                }
                Ok(())
            }

            ConsequenceSource::BCP(key) => {
                log::info!("BCP Consequence: {key}: {}", consequence.literal());
                //
                match self.literal_db.decision_count() {
                    0 => {
                        if self.literal_db.assumption_is_made() {
                            self.literal_db.record_assumption_consequence(consequence);
                        } else {
                            let unit_clause = *consequence.literal();

                            let direct_origin = unsafe { self.clause_db.get_unchecked(&key) }?;

                            let mut origins = HashSet::default();

                            match key {
                                ClauseKey::OriginalUnit(_)
                                | ClauseKey::Binary(_)
                                | ClauseKey::Original(_) => {
                                    origins.insert(*key);
                                }
                                ClauseKey::Addition(_, _) | ClauseKey::AdditionUnit(_) => {}
                            }

                            for literal in direct_origin.literals().filter(|l| *l != &unit_clause) {
                                let literal = literal.negate();
                                let literal_key = ClauseKey::AdditionUnit(literal);

                                match self.clause_db.get(&literal_key) {
                                    Err(_) => {
                                        origins.insert(ClauseKey::OriginalUnit(literal));
                                    }
                                    Ok(db_clause) => {
                                        origins.extend(db_clause.origins());
                                    }
                                }
                            }

                            log::trace!("Origins: {:?}", &origins);

                            self.record_clause(unit_clause, ClauseSource::BCP, None, origins);
                        };
                    }

                    _ => unsafe {
                        self.literal_db.record_consequence_unchecked(consequence);
                    },
                }
                Ok(())
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
        origins: HashSet<ClauseKey>,
    ) -> Result<ClauseKey, err::ClauseDBError> {
        let key = self
            .clause_db
            .store(clause, source, &mut self.atom_db, valuation, origins)?;

        log::info!("Record clause: {key}");

        if let Some(dispatcher) = &self.dispatcher {
            match key {
                ClauseKey::OriginalUnit(_) => {
                    let delta = delta::ClauseDB::Original(key);
                    dispatcher(Dispatch::Delta(delta::Delta::ClauseDB(delta)));
                }

                ClauseKey::AdditionUnit(literal) => match source {
                    ClauseSource::PureUnit => {
                        // TODO: Implement dispatches for free decisions
                    }

                    ClauseSource::BCP => {
                        let delta = delta::ClauseDB::BCP(ClauseKey::AdditionUnit(literal));
                        dispatcher(Dispatch::Delta(delta::Delta::ClauseDB(delta)));
                    }

                    ClauseSource::Resolution => {
                        let delta = delta::ClauseDB::Added(key);
                        dispatcher(Dispatch::Delta(delta::Delta::ClauseDB(delta)));
                    }

                    ClauseSource::Assumption => {
                        // TODO: Deltas for assumptions.
                    }

                    ClauseSource::Original => panic!("!"),
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
                                    ClauseSource::BCP
                                    | ClauseSource::PureUnit
                                    | ClauseSource::Assumption => panic!("!"),
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
