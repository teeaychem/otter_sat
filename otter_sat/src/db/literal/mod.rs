//! A database of literal indexed things.
//!
//! For the moment, this amounts to a stack of all chosen literals.
//!
//! Note, observed consequences which are known to not rest on some choice(s) are stored as unit clauses in the [clause database](crate::db::clause::ClauseDB).

use std::rc::Rc;

use crate::{
    db::LevelIndex,
    dispatch::Dispatch,
    structures::literal::{self, abLiteral},
};

#[doc(hidden)]
mod level;
pub use level::*;

#[allow(dead_code)]
/// A struct abstracting over decision levels.
pub struct LiteralDB {
    /// A stack of levels.
    level_stack: Vec<Level>,
    /// A dispatcher.
    dispatcher: Option<Rc<dyn Fn(Dispatch)>>,
}

impl LiteralDB {
    pub fn new(tx: Option<Rc<dyn Fn(Dispatch)>>) -> Self {
        LiteralDB {
            level_stack: Vec::default(),
            dispatcher: tx,
        }
    }

    /// Notes a choice has been made and pushes a new level to the top of the level stack.
    /// ```rust,ignore
    /// self.literal_db.note_choice(chosen_literal);
    /// ```
    pub fn note_choice(&mut self, choice: abLiteral) {
        self.level_stack.push(Level::new(choice));
    }

    /// The last choice made.
    ///
    /// I.e. the choice of the level at the top of the level stack.
    ///
    /// ```rust,ignore
    /// self.atom_db.drop_value(self.literal_db.last_choice().atom());
    /// ```
    /// # Safety
    /// No check is made to ensure a choice has been made.
    pub unsafe fn last_choice_unchecked(&self) -> abLiteral {
        self.level_stack
            .get_unchecked(self.level_stack.len() - 1)
            .choice()
    }

    /// Consequences of the last choice made.
    ///
    /// I.e. consequences of the choice of the level at the top of the level stack.
    ///
    /// ```rust,ignore
    /// for (source, literal) in literal_db.last_consequences_unchecked().iter().rev() {
    ///    ...
    /// }
    /// ```
    /// # Safety
    /// No check is made to ensure a choice has been made.
    pub fn last_consequences_unchecked(&self) -> &[(literal::Source, abLiteral)] {
        unsafe {
            self.level_stack
                .get_unchecked(self.level_stack.len() - 1)
                .consequences()
        }
    }

    /// Removes the top level from the level stack.
    pub fn forget_last_choice(&mut self) {
        self.level_stack.pop();
    }

    /// Returns true if a choice has been made, false otherwise.
    pub fn choice_made(&self) -> bool {
        !self.level_stack.is_empty()
    }

    /// A count of how many levels are present in the choice stack.
    ///
    /// In other words, a count of how many choices have been made.
    pub fn choice_count(&self) -> LevelIndex {
        self.level_stack.len() as LevelIndex
    }

    /// A mutable borrow of the top level.
    ///
    /// # Safety
    /// No check is made to ensure a choice has been made.
    pub unsafe fn top_mut_unchecked(&mut self) -> &mut Level {
        let last_choice_index = self.level_stack.len().saturating_sub(1);
        self.level_stack.get_unchecked_mut(last_choice_index)
    }
}
