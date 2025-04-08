/*!
Assumptions

# Overview

The [solve_given](GenericContext::solve_given) method performs a solve given some collection of literals to assume. \

Internally, assumptions are handled at the initial stages of the method, and in particular through the [assert_assumptions](GenericContext::assert_assumptions) method.

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
    db::{ClauseKey, atom::AtomValue},
    structures::{
        atom::Atom,
        clause::Clause,
        consequence::AssignmentSource,
        literal::{CLiteral, Literal},
    },
    types::err::{self, ErrorKind},
};

impl<R: rand::Rng + std::default::Default> GenericContext<R> {
    /// Asserts all assumptions recorded in the literal database.
    /// Returns ok if asserting assumptions as successful, and an error otherwise.
    pub fn assert_assumptions(&mut self, assumptions: Vec<CLiteral>) -> Result<(), ErrorKind> {
        if self.trail.decision_is_made() {
            log::error!("! Asserting assumptions while a decision has been made.");
            return Err(ErrorKind::InvalidState);
        }

        // Additional safety notes:
        // Assumptions are stored in the literal database, which is mutated when a fresh decision level is made.
        // For this reason, it is not possible to directly loop over the assumptions, and instead the unsafe `recorded_assumption` method is used to access assumptions by index.
        // This is safe, as no new assumptions will be created when asserting assumptions.
        // Further, the calls to BCP are as safe as can be, as a check is made to ensure the language of the context includes the atom of each assumption added.

        match self.config.stacked_assumptions.value {
            true => {
                for assumption in &assumptions {
                    self.ensure_atom(assumption.atom());

                    let source = self.atom_cells.get_assignment_source(assumption.atom());

                    if source != &AssignmentSource::None {
                        let key = {
                            match source {
                                AssignmentSource::None => panic!(),

                                AssignmentSource::PureLiteral => todo!(),

                                AssignmentSource::Decision => {
                                    panic!("! Decision prior to assumption")
                                }

                                AssignmentSource::BCP(key) => *key,

                                AssignmentSource::Assumption => {
                                    todo!("AssignmentSource::Assumption")
                                }

                                AssignmentSource::Original => ClauseKey::OriginalUnit(-assumption),

                                AssignmentSource::Addition => ClauseKey::AdditionUnit(-assumption),
                            }
                        };

                        self.store_assumption(*assumption);
                        self.note_conflict(key);
                        return Err(ErrorKind::FundamentalConflict);
                    }

                    // # Safety
                    // The atom has been ensured, above.
                    match self.peek_assignment_unchecked(assumption) {
                        AtomValue::NotSet => {
                            self.record_assignment(*assumption, AssignmentSource::Assumption);

                            log::info!("BCP of assumption: {assumption}");
                            // As assumptions are stacked, immediately call BCP.
                            match self.bcp(assumption) {
                                Ok(_) => {}

                                Err(err::BCPError::Conflict(key)) => {
                                    // TODO: Unify re-use of BCP result parsing.
                                    self.note_conflict(key);

                                    return Err(ErrorKind::FundamentalConflict);
                                }

                                Err(err::BCPError::CorruptWatch) => {
                                    panic!("! Corrupt watch with assumptions")
                                }
                            }
                        }

                        AtomValue::Same => log::info!("! Assumption of an atom with same value"),

                        AtomValue::Different => panic!("!"),
                    }
                }

                Ok(())
            }

            false => {
                // All assumption can be made, so push a fresh level.
                // Levels store a single literal, so Top is used to represent the assumptions.

                for literal in assumptions.into_iter() {
                    self.ensure_atom(literal.atom());

                    match self.peek_assignment_unchecked(literal) {
                        AtomValue::NotSet => {
                            self.record_assignment(literal, AssignmentSource::Assumption);
                        }

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

        if !self.trail.assumption_is_made() {
            return assumptions;
        }

        // Atoms are used in place of literals, as a literal and its negation will not appear in the trail.
        // Else, there was a previous conflict to that identified…
        let mut seen_atoms: HashSet<Atom> = HashSet::default();

        // Safe, as the relevant key is kept as proof of unsatisfiability.
        seen_atoms.extend(unsafe { self.clause_db.get_unchecked(&key).atoms() });

        // # Safety
        // Safe, as by the above check some assumption has ben made and so the initial decision level is ≥ 1.
        let assumption_index = unsafe {
            *self
                .trail
                .level_indicies
                .get_unchecked(self.trail.initial_decision_level as usize - 1)
        };

        for (index, literal) in self.trail.literals.iter().enumerate().rev() {
            if seen_atoms.contains(&literal.atom()) {
                // Check for an assumption, as in the case of conflict it will not have been assigned.
                if index <= assumption_index {
                    assumptions.push(*literal);
                }

                match self.atom_cells.get_assignment_source(literal.atom()) {
                    AssignmentSource::Assumption => {} // Handled above

                    AssignmentSource::BCP(key) => {
                        // The method does not require all clauses in a core are preserved, as an assumption is never 'used' during resolution.
                        match self.clause_db.get(key) {
                            Ok(clause) => {
                                for literal in clause.literals() {
                                    seen_atoms.insert(literal.atom());
                                }
                            }

                            Err(_) => {}
                        }
                    }

                    AssignmentSource::Addition
                    | AssignmentSource::Decision
                    | AssignmentSource::Original
                    | AssignmentSource::PureLiteral => {}

                    AssignmentSource::None => panic!("! Missing assignment"),
                }
            }
        }

        assumptions
    }
}
