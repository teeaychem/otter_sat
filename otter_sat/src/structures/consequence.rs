use crate::structures::{
    atom::Atom,
    literal::{self, abLiteral, Literal},
};

pub struct Consequence {
    /// The atom-value bind which must hold, represented as a literal.
    pub literal: abLiteral,

    /// The immediate reason why the atom-value pair must be.
    pub source: literal::Source,
}

impl Consequence {
    /// The bound atom.
    pub fn atom(&self) -> Atom {
        self.literal.atom()
    }

    /// The value the atom is bound to.
    pub fn value(&self) -> bool {
        self.literal.polarity()
    }

    /// The atom-value bind, represented as a literal.
    pub fn literal(&self) -> &abLiteral {
        &self.literal
    }

    /// The (immediate) reason why the atom-value bind must hold.
    pub fn source(&self) -> &literal::Source {
        &self.source
    }
}
