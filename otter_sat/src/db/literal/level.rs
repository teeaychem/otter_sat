use std::borrow::Borrow;

use crate::structures::{
    consequence::Consequence,
    literal::{self, abLiteral},
};

/// A decision level --- the decision and the *observed* consequences of that decision, given prior decisions and observed consequences.
///
/// Note: The consequences relation is reflexive, but no reflexive consequences are observed.
pub struct Level {
    decision: abLiteral,
    consequences: Vec<Consequence>,
}

impl Level {
    /// A new level from some decision, with no recorded consequences.
    pub fn new(decision: abLiteral) -> Self {
        Self {
            decision,
            consequences: vec![],
        }
    }

    /// The decision of a level.
    pub fn decision(&self) -> abLiteral {
        self.decision
    }

    /// The consequences of a level.
    pub fn consequences(&self) -> &[Consequence] {
        &self.consequences
    }

    /// Records a literal consequence of the level from some source.
    ///
    /// No effort is made to check the literal is really a consequence.
    pub fn record_consequence(&mut self, literal: impl Borrow<abLiteral>, source: literal::Source) {
        self.consequences.push(Consequence {
            literal: *literal.borrow(),
            source,
        })
    }
}
