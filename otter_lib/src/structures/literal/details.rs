use std::hash::{Hash, Hasher};

use crate::{db::atom::AtomDB, structures::atom::Atom};

use super::{vbLiteral, Literal};

impl Literal for vbLiteral {
    fn negate(&self) -> Self {
        !*self
    }

    fn new(atom: Atom, polarity: bool) -> Self {
        Self { atom, polarity }
    }

    fn var(&self) -> Atom {
        self.atom
    }

    fn polarity(&self) -> bool {
        self.polarity
    }

    fn canonical(&self) -> super::vbLiteral {
        *self
    }

    fn external_representation(&self, atom_db: &AtomDB) -> String {
        let mut the_string = String::new();
        if !self.polarity {
            the_string.push('-');
        }
        the_string.push_str(atom_db.external_representation(self.atom).as_str());
        the_string
    }
}

impl PartialOrd for vbLiteral {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

/// Literals are ordered by id and polarity on a tie with false < true.
impl Ord for vbLiteral {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        if self.atom == other.atom {
            if self.polarity == other.polarity {
                std::cmp::Ordering::Equal
            } else if self.polarity {
                std::cmp::Ordering::Greater
            } else {
                std::cmp::Ordering::Less
            }
        } else {
            self.atom.cmp(&other.atom)
        }
    }
}

impl PartialEq for vbLiteral {
    fn eq(&self, other: &Self) -> bool {
        self.atom == other.atom && self.polarity == other.polarity
    }
}

impl Hash for vbLiteral {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.atom.hash(state);
        self.polarity.hash(state);
    }
}

impl Eq for vbLiteral {}

impl std::fmt::Display for vbLiteral {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self.polarity {
            true => write!(f, "{}", self.atom),
            false => write!(f, "-{}", self.atom),
        }
    }
}

impl std::ops::Not for vbLiteral {
    type Output = Self;

    fn not(self) -> Self::Output {
        Self {
            atom: self.atom,
            polarity: !self.polarity,
        }
    }
}
