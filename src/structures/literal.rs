use std::sync::atomic::{AtomicUsize, Ordering as AtomicOrdering};

use crate::structures::Solve;

pub type VariableId = u32;

#[derive(Clone, Debug)]
pub struct Variable {
    pub name: String,
    pub id: VariableId,
}

pub type LiteralInt = i64;

#[derive(Clone, Debug)]
pub struct Literal {
    variable: Variable,
    polarity: bool,
}

#[derive(Debug)]
pub enum LiteralError {
    NoVariable,
}

impl std::fmt::Display for Literal {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self.polarity {
            true => write!(f, "{}", self.variable.name),
            false => write!(f, "-{}", self.variable.name),
        }
    }
}

impl Literal {
    pub fn negate(&self) -> Self {
        Literal {
            variable: self.variable.clone(),
            polarity: !self.polarity,
        }
    }

    pub fn new(variable: Variable, polarity: bool) -> Self {
        Literal { variable, polarity }
    }

    pub fn variable(&self) -> &Variable {
        &self.variable
    }

    pub fn polarity(&self) -> bool {
        self.polarity
    }
}

impl PartialOrd for Literal {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Literal {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        if self.variable == other.variable {
            if self.polarity == other.polarity {
                std::cmp::Ordering::Equal
            } else if self.polarity {
                std::cmp::Ordering::Less
            } else {
                std::cmp::Ordering::Greater
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

impl Eq for Literal {}

impl PartialOrd for Variable {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Variable {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.name.cmp(&other.name)
    }
}

impl PartialEq for Variable {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for Variable {}
