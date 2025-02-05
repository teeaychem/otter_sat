use crate::structures::{atom::Atom, clause::ABClause};

use super::{IntLiteral, Literal};

/// The representation of a literal as an atom paired with a boolean.
#[allow(non_camel_case_types)]
#[derive(Clone, Copy, Debug)]
pub struct ABLiteral {
    /// The atom of a literal.
    atom: Atom,

    /// The polarity of a literal.
    polarity: bool,
}

impl Literal for ABLiteral {
    fn new(atom: Atom, polarity: bool) -> Self {
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

    fn canonical(&self) -> super::CLiteral {
        #[cfg(feature = "boolean")]
        return *self;

        #[cfg(not(feature = "boolean"))]
        return self.into();
    }

    fn as_int(&self) -> isize {
        match self.polarity {
            true => self.atom as isize,
            false => -(self.atom as isize),
        }
    }
}

// Traits

impl PartialOrd for ABLiteral {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for ABLiteral {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        if self.atom == other.atom {
            self.polarity.cmp(&other.polarity)
        } else {
            self.atom.cmp(&other.atom)
        }
    }
}

impl PartialOrd<IntLiteral> for ABLiteral {
    fn partial_cmp(&self, other: &IntLiteral) -> Option<std::cmp::Ordering> {
        Some(self.cmp(&ABLiteral::from(other)))
    }
}

impl PartialEq for ABLiteral {
    fn eq(&self, other: &Self) -> bool {
        self.atom == other.atom && self.polarity == other.polarity
    }
}

impl PartialEq<IntLiteral> for ABLiteral {
    fn eq(&self, other: &IntLiteral) -> bool {
        self.atom == other.atom() && self.polarity == other.polarity()
    }
}

impl Eq for ABLiteral {}

impl std::hash::Hash for ABLiteral {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.atom.hash(state);
        self.polarity.hash(state);
    }
}

impl std::fmt::Display for ABLiteral {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self.polarity {
            true => write!(f, "{}", self.atom),
            false => write!(f, "-{}", self.atom),
        }
    }
}

// From

impl From<i16> for ABLiteral {
    fn from(value: i16) -> Self {
        ABLiteral::new(value.unsigned_abs() as Atom, value.is_positive())
    }
}

impl From<i32> for ABLiteral {
    fn from(value: i32) -> Self {
        ABLiteral::new(value.unsigned_abs(), value.is_positive())
    }
}

impl From<&i32> for ABLiteral {
    fn from(value: &i32) -> Self {
        ABLiteral::new(value.unsigned_abs(), value.is_positive())
    }
}

impl TryFrom<i64> for ABLiteral {
    type Error = ();

    fn try_from(value: i64) -> Result<Self, Self::Error> {
        let atom = value.unsigned_abs();
        if atom < Atom::MAX.into() {
            Ok(ABLiteral::new(atom as Atom, value.is_positive()))
        } else {
            Err(())
        }
    }
}

impl TryFrom<isize> for ABLiteral {
    type Error = ();

    fn try_from(value: isize) -> Result<Self, Self::Error> {
        let atom = value.unsigned_abs();
        if Atom::MAX.try_into().is_ok_and(|max| atom < max) {
            Ok(ABLiteral::new(atom as Atom, value.is_positive()))
        } else {
            Err(())
        }
    }
}

// Into

impl Into<ABClause> for ABLiteral {
    fn into(self) -> ABClause {
        vec![self]
    }
}
