use crate::structures::{atom::Atom, clause::cClause};

use super::Literal;

/// The representation of a literal as an atom paired with a boolean.
#[allow(non_camel_case_types)]
#[derive(Clone, Copy, Debug)]
pub struct abLiteral {
    /// The atom of a literal.
    atom: Atom,

    /// The polarity of a literal.
    polarity: bool,
}

impl Literal for abLiteral {
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

    fn canonical(&self) -> super::cLiteral {
        *self
    }

    fn as_int(&self) -> isize {
        match self.polarity {
            true => self.atom as isize,
            false => -(self.atom as isize),
        }
    }
}

// Traits

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

// From

impl From<i16> for abLiteral {
    fn from(value: i16) -> Self {
        abLiteral::new(value.unsigned_abs() as Atom, value.is_positive())
    }
}

impl From<i32> for abLiteral {
    fn from(value: i32) -> Self {
        abLiteral::new(value.unsigned_abs(), value.is_positive())
    }
}

impl From<&i32> for abLiteral {
    fn from(value: &i32) -> Self {
        abLiteral::new(value.unsigned_abs(), value.is_positive())
    }
}

impl TryFrom<i64> for abLiteral {
    type Error = ();

    fn try_from(value: i64) -> Result<Self, Self::Error> {
        let atom = value.unsigned_abs();
        if atom < Atom::MAX.into() {
            Ok(abLiteral::new(atom as Atom, value.is_positive()))
        } else {
            Err(())
        }
    }
}

impl TryFrom<isize> for abLiteral {
    type Error = ();

    fn try_from(value: isize) -> Result<Self, Self::Error> {
        let atom = value.unsigned_abs();
        if Atom::MAX.try_into().is_ok_and(|max| atom < max) {
            Ok(abLiteral::new(atom as Atom, value.is_positive()))
        } else {
            Err(())
        }
    }
}

// Into

impl Into<cClause> for abLiteral {
    fn into(self) -> cClause {
        vec![self]
    }
}
