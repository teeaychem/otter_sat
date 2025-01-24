use std::borrow::Borrow;

use crate::{
    db::ClauseKey,
    structures::{
        atom::Atom,
        literal::{cLiteral, Literal},
    },
};

/// The source of a bind.
#[derive(Clone, Copy, PartialEq, Eq)]
#[allow(clippy::upper_case_acronyms)]
pub enum Source {
    /// A decision was made where the alternative the alternative would make no difference to satisfiability.
    PureLiteral,

    /// A consequence of boolean constraint propagation.
    BCP(ClauseKey),
}

#[derive(Clone)]
pub struct Consequence {
    /// The atom-value bind which must hold, represented as a literal.
    pub literal: cLiteral,

    /// The immediate reason why the atom-value pair must be.
    pub source: Source,
}

impl Consequence {
    /// Creates a consequence from a bind represented as a literal and a source.
    pub fn from(literal: impl Borrow<cLiteral>, source: Source) -> Self {
        Consequence {
            literal: literal.borrow().canonical(),
            source,
        }
    }

    /// Creates a consequence of the given atom bound to the given value due to the given source.
    pub fn from_bind(atom: Atom, value: bool, source: Source) -> Self {
        Consequence {
            literal: cLiteral::fresh(atom, value),
            source,
        }
    }

    /// The bound atom.
    pub fn atom(&self) -> Atom {
        self.literal.atom()
    }

    /// The value the atom is bound to.
    pub fn value(&self) -> bool {
        self.literal.polarity()
    }

    /// The atom-value bind, represented as a literal.
    pub fn literal(&self) -> &cLiteral {
        &self.literal
    }

    /// The (immediate) reason why the atom-value bind must hold.
    pub fn source(&self) -> &Source {
        &self.source
    }
}
