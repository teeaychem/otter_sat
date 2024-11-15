use crate::structures::variable::Variable;

use super::{LiteralStruct, LiteralT};

impl LiteralT for LiteralStruct {
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
}

impl PartialOrd for LiteralStruct {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

/// Literals are ordered by id and polarity on a tie with false < true.
impl Ord for LiteralStruct {
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

impl PartialEq for LiteralStruct {
    fn eq(&self, other: &Self) -> bool {
        self.variable == other.variable && self.polarity == other.polarity
    }
}

impl Eq for LiteralStruct {}

impl std::fmt::Display for LiteralStruct {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self.polarity {
            true => write!(f, "{}", self.variable),
            false => write!(f, "-{}", self.variable),
        }
    }
}

impl std::ops::Not for LiteralStruct {
    type Output = Self;

    fn not(self) -> Self::Output {
        Self {
            variable: self.variable,
            polarity: !self.polarity,
        }
    }
}
