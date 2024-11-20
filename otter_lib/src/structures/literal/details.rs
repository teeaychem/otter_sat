use std::hash::{Hash, Hasher};

use crate::{db::variable::VariableDB, structures::variable::Variable};

use super::{Literal, LiteralT};

impl LiteralT for Literal {
    fn negate(&self) -> Self {
        !*self
    }

    fn new(variable: Variable, polarity: bool) -> Self {
        Self { variable, polarity }
    }

    fn var(&self) -> Variable {
        self.variable
    }

    fn polarity(&self) -> bool {
        self.polarity
    }

    fn canonical(&self) -> super::Literal {
        *self
    }

    fn external_representation(&self, variable_db: &VariableDB) -> String {
        let mut the_string = String::new();
        if !self.polarity {
            the_string.push('-');
        }
        the_string.push_str(variable_db.external_representation(self.variable).as_str());
        the_string
    }
}

impl PartialOrd for Literal {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

/// Literals are ordered by id and polarity on a tie with false < true.
impl Ord for Literal {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        if self.variable == other.variable {
            if self.polarity == other.polarity {
                std::cmp::Ordering::Equal
            } else if self.polarity {
                std::cmp::Ordering::Greater
            } else {
                std::cmp::Ordering::Less
            }
        } else {
            self.variable.cmp(&other.variable)
        }
    }
}

impl PartialEq for Literal {
    fn eq(&self, other: &Self) -> bool {
        self.variable == other.variable && self.polarity == other.polarity
    }
}

impl Hash for Literal {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.variable.hash(state);
        self.polarity.hash(state);
    }
}

impl Eq for Literal {}

impl std::fmt::Display for Literal {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self.polarity {
            true => write!(f, "{}", self.variable),
            false => write!(f, "-{}", self.variable),
        }
    }
}

impl std::ops::Not for Literal {
    type Output = Self;

    fn not(self) -> Self::Output {
        Self {
            variable: self.variable,
            polarity: !self.polarity,
        }
    }
}
