/*!
A database of literal indexed things.

For the moment, this amounts to a stack of all chosen literals.

# Components

## The level stack

A stack of [ADLevel]s, each of which stores a literal, and the observed consequences of that literal (given prior assumptions, decisions and observed consequences).

A first (or bottom) decision level exists only after some assumption or decision has been made.
And, so, in particular, observed consequences which do --- or are known to --- *not* rest on some decision are stored as unit clauses in the [clause database](crate::db::clause::ClauseDB).

# Terminology

The 'top' level is the level of the most recent decision made.
- For example, after deciding 'p' is true and 'q' is false, the top decision level stores the decision to bind 'q' to false.
*/

use crate::{
    config::Config,
    db::LevelIndex,
    structures::{consequence::Consequence, literal::CLiteral},
};

#[doc(hidden)]
mod ad_level;

#[doc(hidden)]
pub mod config;
pub use config::LiteralDBConfig;

pub use ad_level::*;

#[allow(dead_code)]
/// A struct abstracting over assumption/decision levels.
pub struct LiteralDB {
    /// Configuration of the literal database.
    pub config: LiteralDBConfig,

    /// The first level of a decision in a solve.
    /// In other words, any level present *below* the limit contains assumptions.
    pub lowest_decision_level: LevelIndex,

    /// A stack of levels.
    pub level_stack: Vec<ADLevel>,

    /// Stored assumptions.
    pub assumptions: Vec<CLiteral>,
}

impl LiteralDB {
    /// Pushes a fresh level to the top of the level stack with the given decision.
    /// ```rust,ignore
    /// self.literal_db.push_fresh_decision(chosen_literal);
    /// ```
    pub fn push_fresh_decision(&mut self, decision: CLiteral) {
        self.level_stack.push(ADLevel::new(decision));
    }

    /// Pushes a fresh level to the top of the level stack with the given assumption.
    pub fn push_fresh_assumption(&mut self, assumption: CLiteral) {
        self.level_stack.push(ADLevel::new(assumption));
        self.lowest_decision_level += 1;
    }
}

impl LiteralDB {
    /// True if some assumption has been made, false otherwise.
    pub fn assumption_is_made(&self) -> bool {
        self.lowest_decision_level > 0
    }

    /// Stores an assumption to be used (e.g., during the next solve).
    ///
    /// # Soundness
    /// Assumptions must be asserted to take effect.
    /// See [assert_assumptions](crate::context::GenericContext::assert_assumptions).
    pub fn store_assumption(&mut self, assumption: CLiteral) {
        self.assumptions.push(assumption);
    }

    /// The assumptions stored, as a slice.
    pub fn stored_assumptions(&self) -> &[CLiteral] {
        &self.assumptions
    }

    /// Returns the assumption stored at the given index.
    /// Indicies are fixed relative to a single use (e.g. a solve) but should otherwise be considered random.
    ///
    /// # Safety
    /// It is assumed the count of stored assumptions extends to the given index.
    pub unsafe fn stored_assumption(&self, index: usize) -> CLiteral {
        *self.assumptions.get_unchecked(index)
    }

    /// Clears any stored assumptions.
    ///
    /// # Soundness
    /// Does not clear the *valuation* of any assumption.
    pub fn clear_assumptions(&mut self) {
        self.assumptions.clear();
    }
}

impl LiteralDB {
    /// A new [LiteralDB] with local configuration options derived from `config`.
    pub fn new(config: &Config) -> Self {
        LiteralDB {
            config: config.literal_db.clone(),
            lowest_decision_level: 0,
            level_stack: Vec::default(),
            assumptions: Vec::default(),
        }
    }

    /// Returns the lowest decision level.
    /// Zero, if no assumptions has been made, otherwise some higher level.
    pub fn lowest_decision_level(&self) -> LevelIndex {
        self.lowest_decision_level
    }

    /// The decision (or assumption) made at `level`.
    ///
    /// # Soundness
    /// If multiple assumptions are associated with a single level some arbitrary representative assumption is returned.
    ///
    /// # Safety
    /// No check is made to ensure the relevant number of decisions have been made.
    pub unsafe fn decision_unchecked(&self, level: LevelIndex) -> CLiteral {
        self.level_stack.get_unchecked(level as usize).literal()
    }

    /// The decision of the top level.
    ///
    /// ```rust,ignore
    /// self.atom_db.drop_value(self.literal_db.top_decision_unchecked().atom());
    /// ```
    /// # Safety
    /// No check is made to ensure a decision has been made, and so may fail or return an assumption.
    pub unsafe fn top_decision_unchecked(&self) -> CLiteral {
        self.level_stack
            .get_unchecked(self.level_stack.len() - 1)
            .literal()
    }

    /// A slice of the consequences at the given decision level index.
    ///
    /// # Safety
    /// No check is made to ensure a decision has been made, and so may fail or return consequences of an assumption.
    pub fn decision_consequences_unchecked(&self, level: LevelIndex) -> &[Consequence] {
        unsafe {
            self.level_stack
                .get_unchecked(level as usize)
                .consequences()
        }
    }

    /// A slice of the consequences at the top decision level.
    ///
    /// ```rust,ignore
    /// for consequence in literal_db.top_consequences_unchecked().iter().rev() {
    ///    ...
    /// }
    /// ```
    ///
    /// # Safety
    /// No check is made to ensure a decision has been made, and so may fail or return consequences of an assumption.
    pub unsafe fn top_consequences_unchecked(&self) -> &[Consequence] {
        self.level_stack
            .get_unchecked(self.current_level().saturating_sub(1) as usize)
            .consequences()
    }

    /// Removes the top decision level.
    ///
    /// # Soundness
    /// Does not clear the *valuation* of the decision.
    pub fn forget_top_level(&mut self) {
        self.level_stack.pop();
    }

    /// A count of how many decisions have been made.
    /// That is, the count of only those levels containing decisions (as opposed to assumptions).
    ///
    /// In other words, a count of how many decisions have been made.
    pub fn decision_count(&self) -> LevelIndex {
        (self.level_stack.len() as LevelIndex) - self.lowest_decision_level
    }

    /// Returns true if some decision is active, false otherwise (regardless of whether an assumption has been made).
    pub fn decision_is_made(&self) -> bool {
        self.decision_count() > 0
    }

    /// The current level.
    pub fn current_level(&self) -> LevelIndex {
        self.level_stack.len() as LevelIndex
    }

    /// A mutable borrow of the top decision level.
    ///
    /// # Safety
    /// No check is made to ensure a decision has been made.
    pub unsafe fn top_level_unchecked_mut(&mut self) -> &mut ADLevel {
        let top_decision_index = self.level_stack.len().saturating_sub(1);
        self.level_stack.get_unchecked_mut(top_decision_index)
    }
}

impl LiteralDB {
    /// Stores a consequence of the top decision level.
    ///
    /// # Safety
    /// No check is made to ensure a decision has been made.
    pub(super) unsafe fn store_top_consequence_unchecked(&mut self, consequence: Consequence) {
        self.top_level_unchecked_mut()
            .store_consequence(consequence);
    }
}
