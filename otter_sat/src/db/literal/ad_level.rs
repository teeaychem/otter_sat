use crate::structures::{consequence::Consequence, literal::CLiteral};

/// A level storing a literal and its *observed* consequences (given prior assumptions, decisions and observed consequences).
/// As the literal is intended to be a decision or (representative) assumption, the prefix 'AD' is used.
pub struct ADLevel {
    literal: CLiteral,
    consequences: Vec<Consequence>,
}

impl ADLevel {
    /// A new level from some literal, with no recorded consequences.
    pub fn new(literal: CLiteral) -> Self {
        Self {
            literal,
            consequences: vec![],
        }
    }

    /// The literal of a level.
    pub fn literal(&self) -> CLiteral {
        self.literal
    }

    /// The consequences of a level.
    pub fn consequences(&self) -> &[Consequence] {
        &self.consequences
    }

    /// Stores a consequence of the level from some source.
    pub(super) fn store_consequence(&mut self, consequence: Consequence) {
        self.consequences.push(consequence)
    }
}
