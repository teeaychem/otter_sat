//! Implementation details of the [literal trait](Literal) for the [abLiteral] structure.

use crate::{
    db::atom::AtomDB,
    structures::atom::Atom,
    structures::literal::{abLiteral, Literal},
};

impl Literal for abLiteral {
    fn fresh(atom: Atom, polarity: bool) -> Self {
        Self { atom, polarity }
    }

    fn negate(&self) -> Self {
        Self {
            atom: self.atom,
            polarity: !self.polarity,
        }
    }

    fn atom(&self) -> Atom {
        self.atom
    }

    fn polarity(&self) -> bool {
        self.polarity
    }

    fn canonical(&self) -> super::abLiteral {
        *self
    }
}

impl PartialOrd for abLiteral {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for abLiteral {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        if self.atom == other.atom {
            self.polarity.cmp(&other.polarity)
        } else {
            self.atom.cmp(&other.atom)
        }
    }
}

impl PartialEq for abLiteral {
    fn eq(&self, other: &Self) -> bool {
        self.atom == other.atom && self.polarity == other.polarity
    }
}

impl std::hash::Hash for abLiteral {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.atom.hash(state);
        self.polarity.hash(state);
    }
}

impl Eq for abLiteral {}

impl std::fmt::Display for abLiteral {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self.polarity {
            true => write!(f, "{}", self.atom),
            false => write!(f, "-{}", self.atom),
        }
    }
}
