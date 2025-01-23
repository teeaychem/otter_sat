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
    db::DecisionLevelIndex,
    dispatch::Dispatch,
    structures::{consequence::Consequence, literal::abLiteral},
};

#[doc(hidden)]
mod decision_level;
pub use decision_level::*;

#[allow(dead_code)]
/// A struct abstracting over decision levels.
pub struct LiteralDB {
    /// The lower limit to find a decision made during a solve.
    /// In other words, any decision present *below* the limit was made prior to a call to [solve](crate::procedures::solve).
    lower_limit: DecisionLevelIndex,

    /// A stack of decision levels.
    level_stack: Vec<DecisionLevel>,

    /// Assumptions
    assumptions: Vec<abLiteral>,

    /// Consequences of assumptions made
    assumption_consequences: Vec<Consequence>,

    /// A dispatcher.
    dispatcher: Option<Rc<dyn Fn(Dispatch)>>,
}

impl LiteralDB {
    pub fn assumption_is_made(&self) -> bool {
        !self.assumptions.is_empty()
    }

    pub fn assumption_made(&mut self, assumption: abLiteral) {
        self.assumptions.push(assumption);
    }

    pub fn record_assumption_consequence(&mut self, consequence: Consequence) {
        self.assumption_consequences.push(consequence);
    }

    pub fn assumptions(&self) -> &[abLiteral] {
        &self.assumptions
    }

    pub fn assumption_consequences(&self) -> &[Consequence] {
        &self.assumption_consequences
    }

    pub fn clear_assumptions(&mut self) {
        self.assumptions.clear();
        self.assumption_consequences.clear();
    }
}

impl LiteralDB {
    pub fn new(tx: Option<Rc<dyn Fn(Dispatch)>>) -> Self {
        LiteralDB {
            lower_limit: 0,
            level_stack: Vec::default(),
            assumptions: Vec::default(),
            assumption_consequences: Vec::default(),
            dispatcher: tx,
        }
    }

    // TODO: Ensure this is used where appropriate.
    /// Returns the lower limit of the decision stack.
    ///
    /// If greater than zero, decisions made prior to the solve would be cleared by backjumping to a level at or lower to the limit.
    pub fn lower_limit(&self) -> DecisionLevelIndex {
        self.lower_limit
    }

    /// Notes a decision has been made and pushes a new level to the top of the level stack.
    /// ```rust,ignore
    /// self.literal_db.decision_match(chosen_literal);
    /// ```
    pub fn decision_made(&mut self, decision: abLiteral) {
        self.level_stack.push(DecisionLevel::new(decision));
    }

    /// The decision of the given level index.
    ///
    /// # Safety
    /// No check is made to ensure the relevant number of decisions have been made.
    pub unsafe fn decision_unchecked(&self, level: DecisionLevelIndex) -> abLiteral {
        self.level_stack.get_unchecked(level as usize).decision()
    }

    /// The decision of the top level.
    /// ```rust,ignore
    /// self.atom_db.drop_value(self.literal_db.top_decision_unchecked().atom());
    /// ```
    /// # Safety
    /// No check is made to ensure a decision has been made.
    pub unsafe fn top_decision_unchecked(&self) -> abLiteral {
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
            .get_unchecked(self.decision_count().saturating_sub(1) as usize)
            .consequences()
    }

    /// Removes the top decision level.
    ///
    /// Note, this does not mutate any valuation.
    pub fn forget_top_decision(&mut self) {
        self.level_stack.pop();
    }

    /// Returns true if some decision is active, false otherwise.
    pub fn is_decision_made(&self) -> bool {
        !self.level_stack.is_empty()
    }

    /// A count of how many levels are present in the decision stack.
    ///
    /// In other words, a count of how many decisions have been made.
    pub fn decision_count(&self) -> DecisionLevelIndex {
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
    pub(super) unsafe fn record_consequence_unchecked(&mut self, consequence: Consequence) {
        self.top_level_unchecked_mut().push_consequence(consequence);
    }
}
