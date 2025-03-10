/*!
Assumptions

# Overview

Assumptions are *added* to a context through the [add_assumption](GenericContext::add_assumption) method. \
Assumptions are *asserted* for a solve through the [assert_assumption](GenericContext::assert_assumptions) method.

The distinction between adding and asserting assumptions allows for distinct ways of making assumptions (see below).

# Ways of making assumptions

Two ways of making assumptions are supported: Stacked and flat.

# Stacked
A new decision level for each assumption, and immediately applies BCP to an assumption when the level is created.

# Flat
A single decision level for all assumptions and delay BCP until the valuation has been updated with all valuations.
*/

use std::collections::HashSet;

use crate::{
    context::{ContextState, GenericContext},
    db::{atom::AtomValue, consequence_q::QPosition},
    structures::{
        atom::Atom,
        clause::Clause,
        consequence::{Assignment, AssignmentSource},
        literal::{CLiteral, Literal},
    },
    types::err::{self, ErrorKind},
};

impl<R: rand::Rng + std::default::Default> GenericContext<R> {
    /// Asserts all assumptions recorded in the literal database.
    /// Returns ok if asserting assumptions as successful, and an error otherwise.
    pub fn assert_assumptions(&mut self, assumptions: Vec<CLiteral>) -> Result<(), ErrorKind> {
        if self.literal_db.decision_is_made() {
            log::error!("! Asserting assumptions while a decision has been made.");
            return Err(ErrorKind::InvalidState);
        }

        // Additional safety notes:
        // Assumptions are stored in the literal database, which is mutated when a fresh decision level is made.
        // For this reason, it is not possible to directly loop over the assumptions, and instead the unsafe `recorded_assumption` method is used to access assumptions by index.
        // This is safe, as no new assumptions will be created when asserting assumptions.
        // Further, the calls to BCP are as safe as can be, as a check is made to ensure the language of the context includes the atom of each assumption added.

        match self.config.literal_db.stacked_assumptions.value {
            true => {
                for assumption in assumptions {
                    self.ensure_atom(assumption.atom());

                    self.literal_db.push_fresh_assumption(assumption);

                    // # Safety
                    // The atom has been ensured, above.
                    match unsafe {
                        self.atom_db
                            .set_value(assumption, Some(self.literal_db.current_level()))
                    } {
                        AtomValue::NotSet => {
                            // As assumptions are stacked, immediately call BCP.
                            match self.bcp(assumption) {
                                Ok(_) => {}

                                Err(err::BCPError::Conflict(key)) => {
                                    // TODO: Unify re-use of BCP result parsing.

                                    self.state = ContextState::Unsatisfiable(key);

                                    let clause =
                                        unsafe { self.clause_db.get_unchecked(&key).clone() };
                                    self.clause_db.make_callback_unsatisfiable(&clause);

                                    return Err(ErrorKind::FundamentalConflict);
                                }

                                Err(err::BCPError::CorruptWatch) => {
                                    panic!("! Corrupt watch with assumptions")
                                }
                            }
                        }

                        AtomValue::Same => log::info!("! Assumption of an atom with that value"),

                        AtomValue::Different => {
                            return Err(ErrorKind::AssumptionConflict(assumption))
                        }
                    }
                }

                Ok(())
            }

            false => {
                // All assumption can be made, so push a fresh level.
                // Levels store a single literal, so Top is used to represent the assumptions.
                self.literal_db.initial_decision_level += 1;
                self.literal_db
                    .level_indicies
                    .push(self.literal_db.assignments.len());

                for literal in assumptions.into_iter() {
                    self.ensure_atom(literal.atom());

                    self.literal_db.store_assignment(Assignment {
                        literal,
                        source: AssignmentSource::Assumption,
                    });

                    let q_result = self.value_and_queue(
                        literal,
                        QPosition::Back,
                        self.literal_db.current_level(),
                    );

                    match q_result {
                        AtomValue::NotSet => {}

                        AtomValue::Same => log::info!("! Assumption of an atom with that value"),

                        AtomValue::Different => return Err(ErrorKind::AssumptionConflict(literal)),
                    }
                }

                Ok(())
            }
        }
    }

    /**
    Identifies the assumptions used to derive `conflict`.

    Derived from reading MiniSATs `analyzeFinal`.

    The conflict, if it exists, is due to some chain of BCP.
    And, so long as an assumption was used in some part of the chain, it was used to derive the conflict.

    Each part of the chain can be examined by walking through each level, of which at least one must exist if an assumption has been made.
    And, so long as the walk is made backwards a literal is used before it is assumed or derived.
    So, by keeping track of use through a reverse walk, use of an assumption is noted before the assumption is made.
    And, likewise for use of any derived literal, allowing a note to be made on the literals used to derive that (derived) literal.
     */
    pub fn failed_assumpions(&self) -> Vec<CLiteral> {
        let ContextState::Unsatisfiable(key) = self.state else {
            panic!("! Unsatisfiability required to determine failed assumptions");
        };

        let mut assumptions: Vec<CLiteral> = Vec::default();

        if !self.literal_db.assumption_is_made() {
            return assumptions;
        }

        // Atoms are used in place of literals, as a literal and it's negation will not appear in the trail.
        // Else, there was a previous conflict to that identified…
        let mut used_atoms: HashSet<Atom> = HashSet::default();

        // Safe, as the relevant key is kept as proof of unsatisfiability.
        for literal in unsafe { self.clause_db.get_unchecked(&key).literals() } {
            used_atoms.insert(literal.atom());
        }

        for level in (0..self.literal_db.current_level()).rev() {
            // Safe, as the level is bound by the current_level method.
            let assignments = unsafe { self.literal_db.assignments_at_unchecked(level) };

            for assignment in assignments.iter().rev() {
                if used_atoms.contains(&assignment.literal().atom()) {
                    match assignment.source() {
                        AssignmentSource::Assumption => {
                            assumptions.push(*assignment.literal());
                        }

                        AssignmentSource::BCP(key) => {
                            // The method does not require all clauses in a core are preserved, as an assumption is never 'used' during resolution.
                            match self.clause_db.get(key) {
                                Ok(clause) => {
                                    for literal in clause.literals() {
                                        used_atoms.insert(literal.atom());
                                    }
                                }

                                Err(_) => {}
                            }
                        }

                        AssignmentSource::Decision | AssignmentSource::PureLiteral => {}
                    }
                }
            }
        }

        assumptions
    }
}
