use std::borrow::Borrow;

use crate::structures::literal::{self, abLiteral};

/// A choice/decision level.
///
/// In other words, a choice and the *observed* consequences of that choice, given prior choices and observed consequences.
///
/// Note: The consequences relation is reflexive, but no reflexive consequences are observed.
pub struct Level {
    choice: abLiteral,
    consequences: Vec<(literal::Source, abLiteral)>,
}

impl Level {
    /// A new level from some choice, with no recorded consequences.
    pub fn new(choice: abLiteral) -> Self {
        Self {
            choice,
            consequences: vec![],
        }
    }

    /// The choice of a level.
    pub fn choice(&self) -> abLiteral {
        self.choice
    }

    /// The consequences of a level.
    pub fn consequences(&self) -> &[(literal::Source, abLiteral)] {
        &self.consequences
    }

    /// Records a literal consequence of the level from some source.
    ///
    /// No effort is made to check the literal is really a consequence.
    pub fn record_consequence(&mut self, literal: impl Borrow<abLiteral>, source: literal::Source) {
        self.consequences.push((source, *literal.borrow()))
    }
}
