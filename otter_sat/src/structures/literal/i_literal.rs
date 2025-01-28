use crate::structures::atom::Atom;

use super::{abLiteral, cLiteral, Literal};

/// The representation of a literal as an atom paired with a boolean.
#[allow(non_camel_case_types)]
pub type iLiteral = i32;

impl Literal for iLiteral {
    fn new(atom: Atom, polarity: bool) -> Self {
        match polarity {
            true => atom as iLiteral,
            false => -(atom as iLiteral),
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

    fn canonical(&self) -> super::cLiteral {
        cLiteral::new(self.atom(), self.polarity())
    }

    fn as_int(&self) -> isize {
        *self as isize
    }
}

// From

impl From<abLiteral> for iLiteral {
    fn from(value: abLiteral) -> Self {
        let atom = value.atom();
        match value.polarity() {
            true => atom as iLiteral,
            false => -(atom as iLiteral),
        }
    }
}
