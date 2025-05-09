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

  - [The trail](crate::db::trail)
    + Details of assignments made, such as the current valuation and the source of each assignment.
*/

pub mod activity;
pub mod clause;
mod keys;
pub use keys::*;
pub mod trail;
pub mod watches;

use std::collections::HashSet;

use crate::{
    atom_cells::cell::ResolutionFlag,
    config::MinimizationCriteria,
    context::GenericContext,
    structures::{
        clause::ClauseSource,
        consequence::AssignmentSource,
        literal::{CLiteral, Literal},
    },
};

/// The index of a [assumption or decision level](crate::structures::atom).
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
    pub fn record_assignment(&mut self, literal: CLiteral, source: AssignmentSource) {
        // Note if the literal is proven in order to set a flag in the atom cell.
        let mut proven_literal = false;

        match source {
            AssignmentSource::None => panic!("! Assignment without source"),

            AssignmentSource::Pure => {
                let premises = HashSet::default();
                // Making a free decision is not supported after some other (non-free) decision has been made.
                if !self.trail.assumption_is_made() && !self.trail.decision_is_made() {
                    self.clause_db.store(
                        literal,
                        ClauseSource::Unit,
                        &mut self.atom_cells,
                        &mut self.watches,
                        premises,
                    );

                    proven_literal = true;
                } else {
                    panic!("! Origins")
                }
            }

            AssignmentSource::BCP(key) => {
                log::info!("BCP Consequence: {key}: {}", literal);
                //
                if !self.trail.assumption_is_made() {
                    match self.trail.decision_count() {
                        0 => {
                            if !self.trail.assumption_is_made() {
                                let unit_clause = literal;

                                let mut premises = HashSet::default();
                                premises.insert(key);

                                self.clause_db.lock_addition_clause(key);

                                self.clause_db.store(
                                    unit_clause,
                                    ClauseSource::BCP,
                                    &mut self.atom_cells,
                                    &mut self.watches,
                                    premises,
                                );
                            };

                            proven_literal = true;
                        }

                        _decisions_made => {}
                    }
                }

                self.trail.write_literal(literal)
            }

            AssignmentSource::Addition => {
                self.trail.write_literal(literal);
                if !self.trail.assumption_is_made() && !self.trail.decision_is_made() {
                    proven_literal = true;
                }
            }

            AssignmentSource::Original => {
                self.trail.write_literal(literal);
                proven_literal = true;
            }

            AssignmentSource::Decision => {
                self.trail.level_indices.push(self.trail.assignments.len());
                self.trail.write_literal(literal)
            }

            AssignmentSource::Assumption => {
                self.write_assumption(literal);
            }
        }

        let cell = self.atom_cells.get_cell_mut(literal.atom());

        cell.value = Some(literal.polarity());
        cell.source = source;
        cell.level = Some(self.trail.level());

        if proven_literal {
            match self.config.minimization.value {
                MinimizationCriteria::Recursive | MinimizationCriteria::Proven => {
                    cell.resolution_flag = ResolutionFlag::Proven;
                }

                MinimizationCriteria::None => {}
            }
        }
    }

    /// Writes an assumption to the context.
    pub fn write_assumption(&mut self, literal: CLiteral) {
        // The only issue is whether to introduce a fresh level for the assumption…
        if self.config.stacked_assumptions.value
            || self.trail.assignments.last().is_none()
            || self.trail.assignments.last().is_some_and(|assignment| {
                self.atom_cells.get_assignment_source(assignment.atom())
                    != &AssignmentSource::Assumption
            })
        {
            self.trail.initial_decision_level += 1;
            self.trail.level_indices.push(self.trail.assignments.len());
        }

        self.trail.write_literal(literal);
    }
}
