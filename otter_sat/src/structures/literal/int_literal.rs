use crate::structures::atom::Atom;

use super::{ABLiteral, Literal};

/// The representation of a literal as an atom paired with a boolean.
pub type IntLiteral = i32;

impl Literal for IntLiteral {
    fn new(atom: Atom, polarity: bool) -> Self {
        match polarity {
            true => atom as IntLiteral,
            false => -(atom as IntLiteral),
        }
    }

    fn negate(&self) -> Self {
        -self
    }

    fn atom(&self) -> Atom {
        self.unsigned_abs()
    }

    fn polarity(&self) -> bool {
        self.is_positive()
    }

    fn canonical(&self) -> super::CLiteral {
        #[cfg(feature = "boolean")]
        return ABLiteral::new(self.atom(), self.polarity());

        #[cfg(not(feature = "boolean"))]
        return *self;
    }

    fn as_int(&self) -> isize {
        *self as isize
    }
}

// From

impl From<ABLiteral> for IntLiteral {
    fn from(value: ABLiteral) -> Self {
        let atom = value.atom();
        match value.polarity() {
            true => atom as IntLiteral,
            false => -(atom as IntLiteral),
        }
    }
}

impl From<&ABLiteral> for IntLiteral {
    fn from(value: &ABLiteral) -> Self {
        let atom = value.atom();
        match value.polarity() {
            true => atom as IntLiteral,
            false => -(atom as IntLiteral),
        }
    }
}

impl PartialOrd<ABLiteral> for IntLiteral {
    fn partial_cmp(&self, other: &ABLiteral) -> Option<std::cmp::Ordering> {
        Some(self.cmp(&IntLiteral::from(other)))
    }
}

impl PartialEq<ABLiteral> for IntLiteral {
    fn eq(&self, other: &ABLiteral) -> bool {
        self.atom() == other.atom() && self.polarity() == other.polarity()
    }
}
