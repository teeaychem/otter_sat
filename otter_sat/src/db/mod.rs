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

  - [The assignment database](crate::db::atom)
    + Details of assignments made, such as the current valuation and the source of each assignment.
*/

pub mod atom;
pub mod clause;
mod keys;
pub use keys::*;
pub mod trail;
pub mod watches;

use std::collections::HashSet;

use crate::{
    context::GenericContext,
    structures::{
        clause::ClauseSource,
        consequence::{Assignment, AssignmentSource},
        literal::{CLiteral, Literal},
    },
};

/// The index of a [assumption or decision level](crate::db::atom).
///
/// As there can only be as many decisions as there are atoms, this aliases the implementation of atoms.
pub type LevelIndex = crate::structures::atom::Atom;

/// Canonical methods to record literals and clauses to the context.
impl<R: rand::Rng + std::default::Default> GenericContext<R> {
    /// Records a literal in the appropriate database.
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
    pub fn record_assignment(&mut self, assignment: Assignment) {
        match assignment.source() {
            AssignmentSource::PureLiteral => {
                let premises = HashSet::default();
                // Making a free decision is not supported after some other (non-free) decision has been made.
                if !self.trail.decision_is_made() {
                    self.clause_db.store(
                        *assignment.literal(),
                        ClauseSource::PureUnit,
                        &self.valuation,
                        &mut self.atom_db,
                        &mut self.watches,
                        premises,
                    );
                } else {
                    panic!("! Origins")
                }
            }

            AssignmentSource::BCP(key) => {
                log::info!("BCP Consequence: {key}: {}", assignment.literal());
                //
                match self.trail.decision_count() {
                    0 => {
                        if !self.trail.assumption_is_made() {
                            let unit_clause = assignment.literal();

                            let mut premises = HashSet::default();
                            premises.insert(*key);

                            self.clause_db.note_use(*key);

                            self.clause_db.store(
                                *unit_clause,
                                ClauseSource::BCP,
                                &self.valuation,
                                &mut self.atom_db,
                                &mut self.watches,
                                premises,
                            );
                        };
                    }

                    _ => {}
                }

                self.trail.store_assignment(*assignment.literal())
            }

            AssignmentSource::Addition | AssignmentSource::Original => {
                self.trail.store_assignment(*assignment.literal())
            }

            AssignmentSource::Decision => {
                self.trail.level_indicies.push(self.trail.literals.len());
                self.trail.store_assignment(*assignment.literal())
            }

            AssignmentSource::Assumption => {
                self.store_assumption(*assignment.literal());
            }
        }

        let literal = assignment.literal;
        let atom = literal.atom();
        let value = literal.polarity();

        *unsafe { self.valuation.get_unchecked_mut(atom as usize) } = Some(value);

        *unsafe { self.atom_db.atom_level_map.get_unchecked_mut(atom as usize) } =
            Some(self.trail.level());

        self.atom_cells.set_valuation(
            assignment.atom(),
            Some(assignment.value()),
            Some(assignment.clone()),
        );
    }

    pub fn store_assumption(&mut self, literal: CLiteral) {
        if self.config.stacked_assumptions.value
            || self.trail.literals.last().is_none_or(|a| {
                let Some(assignment) = self.atom_cells.get_assignment(a.atom()) else {
                    panic!("! Missing assignment");
                };

                assignment.source != AssignmentSource::Assumption
            })
        {
            self.trail.initial_decision_level += 1;
            self.trail.level_indicies.push(self.trail.literals.len());
        }

        self.trail.store_assignment(literal);
    }
}
