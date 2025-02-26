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
    db::consequence_q::QPosition,
    structures::{
        atom::Atom,
        clause::Clause,
        consequence::AssignmentSource,
        literal::{CLiteral, Literal},
    },
    types::err::ErrorKind,
};

impl<R: rand::Rng + std::default::Default> GenericContext<R> {
    /// Adds `assumption` to the context.
    ///
    /// Note, to ensure the assumption is asserted, [assert_assumptions](GenericContext::assert_assumptions) should be called.
    pub fn add_assumption(&mut self, assumption: CLiteral) -> Result<(), ErrorKind> {
        self.ensure_atom(assumption.atom());
        self.literal_db.store_assumption(assumption);
        Ok(())
    }

    /// Asserts all assumptions recorded in the literal database.
    /// Returns ok if asserting assumptions as successful, and an error otherwise.
    ///
    /// # Safety
    /// Calls to [BCP](GenericContext::bcp) are made.
    pub unsafe fn assert_assumptions(&mut self) -> Result<(), ErrorKind> {
        if self.literal_db.decision_is_made() {
            log::error!("! Asserting assumptions while a decision has been made.");
            return Err(ErrorKind::InvalidState);
        }

        if self.literal_db.stored_assumptions().is_empty() {
            return Ok(());
        }

        // Additional safety notes:
        // Assumptions are stored in the literal database, which is mutated when a fresh decision level is made.
        // For this reason, it is not possible to directly loop over the assumptions, and instead the unsafe `recorded_assumption` method is used to access assumptions by index.
        // This is safe, as no new assumptions will be created when asserting assumptions.
        // Further, the calls to BCP are as safe as can be, as a check is made to ensure the language of the context includes the atom of each assumption added.

        let assumption_count = self.literal_db.stored_assumptions().len();

        match self.config.literal_db.stacked_assumptions.value {
            true => {
                for index in 0..assumption_count {
                    let assumption = self.literal_db.stored_assumption(index);
                    let Ok(_) = self.atom_db.set_value(
                        assumption.atom(),
                        assumption.polarity(),
                        Some(self.literal_db.current_level() + 1),
                    ) else {
                        return Err(ErrorKind::SpecificValuationConflict(assumption));
                    };

                    // Assumption can be made, so push a fresh level.
                    self.literal_db.push_fresh_assumption(assumption);

                    // As assumptions are stacked, immediately call BCP.
                    let Ok(_) = self.bcp(assumption) else {
                        return Err(ErrorKind::SpecificValuationConflict(assumption));
                    };
                }

                Ok(())
            }

            false => {
                // All assumption can be made, so push a fresh level.
                // Levels store a single literal, so Top is used to represent the assumptions.
                let an_assumption = self.literal_db.stored_assumption(0);

                self.literal_db.push_fresh_assumption(an_assumption);

                for index in 0..assumption_count {
                    let assumption = self.literal_db.stored_assumption(index);
                    let Ok(_) = self.value_and_queue(
                        assumption,
                        QPosition::Back,
                        self.literal_db.current_level(),
                    ) else {
                        return Err(ErrorKind::SpecificValuationConflict(assumption));
                    };
                }

                Ok(())
            }
        }
    }

    /// Identifies the assumptions used to derive `conflict`.
    /*
    The implementation is derived from reading MiniSATs `analyzeFinal`.

    The conflict, if it exists, is due to some chain of BCP.
    And, so long as an assumption was used in some part of the chain, it was used to derive the conflict.

    Each part of the chain can be examined by walking through each level, of which at least one must exist if an assumption has been made.
    And, so long as the walk is made backwards a literal is used before it is assumed or derived.
    So, by keeping track of use through a reverse walk, use of an assumption is noted before the assumption is made.
    And, likewise for use of any derived literal, allowing a note to be made on the literals used to derive that (derived) literal.

    Note, this does not require all clauses in a core are preserved, as an assumption is never 'used' during resolution.

    In the implementation, atoms are used in place of literals, as a literal and it's negation will not appear in the trail (else there was a previous conflict to that identified).
     */
    pub fn failed_assumpions(&self) -> Vec<CLiteral> {
        let ContextState::Unsatisfiable(key) = self.state else {
            panic!("! Unsatisfiability required to determine failed assumptions");
        };

        let mut assumptions: Vec<CLiteral> = Vec::default();

        if !self.literal_db.assumption_is_made() {
            return assumptions;
        }

        let mut used_atoms: HashSet<Atom> = HashSet::default();
        // Safe, as the relevant key is kept as proof of unsatisfiability.
        for literal in unsafe { self.clause_db.get_unchecked(&key).unwrap().literals() } {
            used_atoms.insert(literal.atom());
        }

        for level in (0..self.literal_db.current_level()).rev() {
            // Safe, as the level is bound by the current_level method.
            let assignments = unsafe { self.literal_db.assignments_unchecked(level) };

            for assignment in assignments.iter().rev() {
                if used_atoms.contains(&assignment.literal().atom()) {
                    match assignment.source() {
                        AssignmentSource::Assumption => {
                            assumptions.push(*assignment.literal());
                        }

                        AssignmentSource::BCP(key) => {
                            //
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
