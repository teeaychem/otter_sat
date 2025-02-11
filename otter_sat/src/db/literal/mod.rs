//! A database of literal indexed things.
//!
//! For the moment, this amounts to a stack of all chosen literals.
//!
//! # Components
//!
//! ## The decision level stack
//!
//! A stack of [DecisionLevel]s, each of which records a decision made, and the observed consequences of that decision (within the context (of the valuation) the decision was made with respect to).
//!
//! A first (or bottom) decision level exists only after some decision has been made.
//! And, so, in particular, observed consequences which do --- or are known to --- *not* rest on some decision are stored as unit clauses in the [clause database](crate::db::clause::ClauseDB).
//!
//! # Terminology
//!
//! The 'top' level is the level of the most recent decision made.
//! - For example, after deciding 'p' is true and 'q' is false, the top decision level records the decision to bind 'q' to false.
//!

use std::rc::Rc;

use crate::{
    config::Config,
    db::DecisionLevelIndex,
    dispatch::Dispatch,
    structures::{consequence::Consequence, literal::CLiteral},
};

pub mod config;
#[doc(hidden)]
mod decision_level;
use config::LiteralDBConfig;
pub use decision_level::*;

#[allow(dead_code)]
/// A struct abstracting over decision levels.
pub struct LiteralDB {
    pub config: LiteralDBConfig,

    /// The lower limit to find a decision made during a solve.
    /// In other words, any decision present *below* the limit was made prior to a call to [solve](crate::procedures::solve).
    pub lower_limit: DecisionLevelIndex,

    /// A stack of decision levels.
    pub level_stack: Vec<DecisionLevel>,

    /// Assumptions
    pub assumptions: Vec<CLiteral>,

    /// A dispatcher.
    pub dispatcher: Option<Rc<dyn Fn(Dispatch)>>,
}

impl LiteralDB {
    /// Pushes a fresh level to the top of the level stack with the given decision.
    /// ```rust,ignore
    /// self.literal_db.push_fresh_decision(chosen_literal);
    /// ```
    pub fn push_fresh_decision(&mut self, decision: CLiteral) {
        self.level_stack.push(DecisionLevel::new(decision));
    }

    /// Pushes a fresh level to the top of the level stack with the given assumption.
    pub fn push_fresh_assumption(&mut self, assumption: CLiteral) {
        self.level_stack.push(DecisionLevel::new(assumption));
        self.lower_limit += 1;
    }
}

impl LiteralDB {
    pub fn assumption_is_made(&self) -> bool {
        !self.assumptions.is_empty()
    }

    pub fn assumption_is_asserted(&self) -> bool {
        self.lower_limit > 0
    }

    /// Notes, but does not assert, the given assumption is to be used during a solve.
    pub fn note_assumption(&mut self, assumption: CLiteral) {
        self.assumptions.push(assumption);
    }

    pub fn recorded_assumptions(&self) -> &[CLiteral] {
        &self.assumptions
    }

    /// Returns the recorded assumption at the given index.
    /// # Safety
    /// It is assumed the count of recorded assumptions extends to the given index.
    pub unsafe fn recorded_assumption(&self, index: usize) -> CLiteral {
        *self.assumptions.get_unchecked(index)
    }

    pub fn clear_assumptions(&mut self) {
        self.assumptions.clear();
    }
}

impl LiteralDB {
    pub fn new(config: &Config, dispatcher: Option<Rc<dyn Fn(Dispatch)>>) -> Self {
        LiteralDB {
            config: config.literal_db.clone(),
            lower_limit: 0,
            level_stack: Vec::default(),
            assumptions: Vec::default(),
            dispatcher,
        }
    }

    // TODO: Ensure this is used where appropriate.
    /// Returns the lower limit of the decision stack.
    ///
    /// If greater than zero, decisions made prior to the solve would be cleared by backjumping to a level at or lower to the limit.
    pub fn lower_limit(&self) -> DecisionLevelIndex {
        self.lower_limit
    }

    /// The decision of the given level index.
    ///
    /// # Safety
    /// No check is made to ensure the relevant number of decisions have been made.
    pub unsafe fn decision_unchecked(&self, level: DecisionLevelIndex) -> CLiteral {
        self.level_stack.get_unchecked(level as usize).decision()
    }

    /// The decision of the top level.
    /// ```rust,ignore
    /// self.atom_db.drop_value(self.literal_db.top_decision_unchecked().atom());
    /// ```
    /// # Safety
    /// No check is made to ensure a decision has been made.
    pub unsafe fn top_decision_unchecked(&self) -> CLiteral {
        self.level_stack
            .get_unchecked(self.level_stack.len() - 1)
            .decision()
    }

    /// A slice of the consequences at the given decision level index.
    ///
    /// # Safety
    /// No check is made to ensure a decision has been made.
    pub fn decision_consequences_unchecked(&self, level: DecisionLevelIndex) -> &[Consequence] {
        unsafe {
            self.level_stack
                .get_unchecked(level as usize)
                .consequences()
        }
    }

    /// A slice of the consequences at the top decision level.
    /// ```rust,ignore
    /// for consequence in literal_db.top_consequences_unchecked().iter().rev() {
    ///    ...
    /// }
    /// ```
    /// # Safety
    /// No check is made to ensure a decision has been made.
    pub unsafe fn top_consequences_unchecked(&self) -> &[Consequence] {
        self.level_stack
            .get_unchecked(self.decision_level().saturating_sub(1) as usize)
            .consequences()
    }

    /// Removes the top decision level.
    ///
    /// Note, this does not mutate any valuation.
    pub fn forget_top_decision(&mut self) {
        self.level_stack.pop();
    }

    /// Returns true if some decision is active, false otherwise.
    pub fn decision_is_made(&self) -> bool {
        self.decision_count() > 0
    }

    /// A count of how many levels are present in the decision stack.
    ///
    /// In other words, a count of how many decisions have been made.
    pub fn decision_count(&self) -> DecisionLevelIndex {
        (self.level_stack.len() as DecisionLevelIndex) - self.lower_limit
    }

    pub fn decision_level(&self) -> DecisionLevelIndex {
        self.level_stack.len() as DecisionLevelIndex
    }

    /// A mutable borrow of the top decision level.
    ///
    /// # Safety
    /// No check is made to ensure a decision has been made.
    pub unsafe fn top_level_unchecked_mut(&mut self) -> &mut DecisionLevel {
        let top_decision_index = self.level_stack.len().saturating_sub(1);
        self.level_stack.get_unchecked_mut(top_decision_index)
    }
}

impl LiteralDB {
    /// Records a consequence to the top decision level.
    ///
    /// # Safety
    /// No check is made to ensure a decision has been made.
    pub(super) unsafe fn record_top_consequence_unchecked(&mut self, consequence: Consequence) {
        self.top_level_unchecked_mut().push_consequence(consequence);
    }
}
