/*!
Databases for holding information relevant to a solve.

  - [The clause database](crate::db::clause)
    + A collection of clauses, each indexed by a clause key. \
      From an external perspective there are two important kinds of clause:
      * Original clauses \
        Original clauses are added to the context from some external source (e.g. directly or through some DIMACS file). \
        The collection of original clauses together with the collection of original literals are the CNF formula 𝐅 whose satisfiability may be determined.
      * Added clauses \
        Clauses added to the context by some procedure (e.g. via resolution).
        Every added clause is a consequence of the collection of original clauses.

  - [The literal database](crate::db::literal)
    + The literal database handled structures who primary
      * The decision stack
  - [The atom database](crate::db::atom)
    + Properties of atoms.
      * Valuation
      * Watch database
- [Consequence queue](crate::db::consequence_q)
*/

pub mod atom;
pub mod clause;
pub mod consequence_q;
mod keys;
pub use keys::*;
pub mod literal;

use std::collections::HashSet;

use crate::{
    context::GenericContext,
    structures::{
        clause::ClauseSource,
        consequence::{Assignment, AssignmentSource},
    },
};

/// The index of a [decision level](crate::db::literal).
pub type LevelIndex = u32;

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
    /// # Premises
    /// If a propagation occurs without any decision having been made, then the valuation must conflict with each other literal in the clause.
    /// So, the origin of the unit is the clause used and each literal, and the literals are easily identified by examining the clause.
    ///
    /// # Safety
    /// If the source of the consequence references a clause stored by a key, the clause must be present in the clause database.
    pub unsafe fn record_consequence(&mut self, consequence: Assignment) {
        match consequence.source() {
            AssignmentSource::PureLiteral => {
                let premises = HashSet::default();
                // Making a free decision is not supported after some other (non-free) decision has been made.
                if !self.literal_db.decision_is_made() {
                    self.clause_db.store(
                        *consequence.literal(),
                        ClauseSource::PureUnit,
                        &mut self.atom_db,
                        premises,
                    );
                } else {
                    panic!("! Origins")
                }
            }

            AssignmentSource::BCP(key) => {
                log::info!("BCP Consequence: {key}: {}", consequence.literal());
                //
                match self.literal_db.decision_count() {
                    0 => {
                        if !self.literal_db.assumption_is_made() {
                            let unit_clause = consequence.literal();

                            let mut premises = HashSet::default();
                            premises.insert(*key);

                            self.clause_db.note_use(*key);

                            self.clause_db.store(
                                *unit_clause,
                                ClauseSource::BCP,
                                &mut self.atom_db,
                                premises,
                            );
                        };

                        self.literal_db.store_assignment(consequence)
                    }

                    _ => self.literal_db.store_assignment(consequence),
                }
            }

            AssignmentSource::Decision | AssignmentSource::Assumption => {
                panic!("! Store of a non-consequence")
            }
        }
    }
}
