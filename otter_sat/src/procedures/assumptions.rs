//! Assumptions
//!
//! # Overview
//!
//! Assumptions are *added* to a context through the [add_assumption](GenericContext::add_assumption) method. \
//! Assumptions are *asserted* for a solve through the [assert_assumption](GenericContext::assert_assumptions) method.
//!
//! The distinction between adding and asserting assumptions allows for distinct ways of making assumptions (see below).
//!
//! # Ways of making assumptions
//!
//! Two ways of making assumptions are supported: Stacked and flat.
//!
//! # Stacked
//! A new decision level for each assumption, and immediately applies BCP to an assumption when the level is created.
//!
//! # Flat
//! A single decision level for all assumptions and delay BCP until the valuation has been updated with all valuations.

use crate::{
    context::GenericContext,
    db::consequence_q::QPosition,
    structures::literal::{CLiteral, Literal},
    types::err::ErrorKind,
};

impl<R: rand::Rng + std::default::Default> GenericContext<R> {
    pub fn add_assumption(&mut self, assumption: CLiteral) -> Result<(), ErrorKind> {
        self.ensure_atom(assumption.atom());
        self.literal_db.note_assumption(assumption);
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

        if self.literal_db.recorded_assumptions().is_empty() {
            return Ok(());
        }

        // Additional safety notes:
        // Assumptions are stored in the literal database, which is mutated when a fresh decision level is made.
        // For this reason, it is not possible to directly loop over the assumptions, and instead the unsafe `recorded_assumption` method is used to access assumptions by index.
        // This is safe, as no new assumptions will be created when asserting assumptions.
        // Further, the calls to BCP are as safe as can be, as a check is made to ensure the language of the context includes the atom of each assumption added.

        let assumption_count = self.literal_db.recorded_assumptions().len();

        match self.config.literal_db.stacked_assumptions {
            true => {
                for index in 0..assumption_count {
                    let assumption = self.literal_db.recorded_assumption(index);
                    let Ok(_) = self.atom_db.set_value(
                        assumption.atom(),
                        assumption.polarity(),
                        Some(self.literal_db.decision_level() + 1),
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
                let an_assumption = self.literal_db.recorded_assumption(0);

                self.literal_db.push_fresh_assumption(an_assumption);

                for index in 0..assumption_count {
                    let assumption = self.literal_db.recorded_assumption(index);
                    let Ok(_) = self.value_and_queue(
                        assumption,
                        QPosition::Back,
                        self.literal_db.decision_level(),
                    ) else {
                        return Err(ErrorKind::SpecificValuationConflict(assumption));
                    };
                }

                Ok(())
            }
        }
    }
}
