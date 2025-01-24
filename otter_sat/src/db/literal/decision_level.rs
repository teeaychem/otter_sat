use crate::structures::{consequence::Consequence, literal::cLiteral};

/// A decision level --- the decision and the *observed* consequences of that decision, given prior decisions and observed consequences.
///
/// Note: The consequences relation is reflexive, but no reflexive consequences are observed.
pub struct DecisionLevel {
    decision: cLiteral,
    consequences: Vec<Consequence>,
}

impl DecisionLevel {
    /// A new level from some decision, with no recorded consequences.
    pub fn new(decision: cLiteral) -> Self {
        Self {
            decision,
            consequences: vec![],
        }
    }

    /// The decision of a level.
    pub fn decision(&self) -> cLiteral {
        self.decision
    }

    /// The consequences of a level.
    pub fn consequences(&self) -> &[Consequence] {
        &self.consequences
    }

    /// Records a literal consequence of the level from some source.
    ///
    /// No effort is made to check the literal is really a consequence.
    pub(super) fn push_consequence(&mut self, consequence: Consequence) {
        self.consequences.push(consequence)
    }
}
