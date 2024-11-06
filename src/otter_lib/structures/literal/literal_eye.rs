use crate::structures::variable::VariableId;

use super::{LiteralEye, LiteralStruct, LiteralTrait};

impl LiteralTrait for LiteralEye {
    fn negate(&self) -> Self {
        LiteralEye(!self.0)
    }

    fn new(variable_id: VariableId, polarity: bool) -> Self {
        match polarity {
            true => LiteralEye(variable_id as isize),
            false => LiteralEye(!(variable_id as isize)),
        }
    }

    fn v_id(&self) -> VariableId {
        match self.polarity() {
            true => self.0 as VariableId,
            false => !self.0 as VariableId,
        }
    }

    fn polarity(&self) -> bool {
        self.0 >= 0
    }

    fn index(&self) -> usize {
        match self.polarity() {
            true => self.0 as usize,
            false => !self.0 as usize,
        }
    }

    fn canonical(&self) -> super::Literal {
        // self.as_struct()
        *self
    }
}

impl LiteralEye {
    pub fn as_struct(&self) -> LiteralStruct {
        LiteralStruct {
            v_id: self.0 as VariableId,
            polarity: self.polarity(),
        }
    }
}

/// literals are ordered by id and polarity on a tie with false < true.
impl PartialOrd for LiteralEye {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }

    fn lt(&self, other: &Self) -> bool {
        match self.0.abs().cmp(&other.0.abs()) {
            std::cmp::Ordering::Equal => self.0.lt(&other.0),
            std::cmp::Ordering::Less => true,
            std::cmp::Ordering::Greater => false,
        }
    }

    fn gt(&self, other: &Self) -> bool {
        match self.0.abs().cmp(&other.0.abs()) {
            std::cmp::Ordering::Equal => self.0.gt(&other.0),
            std::cmp::Ordering::Less => false,
            std::cmp::Ordering::Greater => true,
        }
    }
}

impl Ord for LiteralEye {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match self.0.abs().cmp(&other.0.abs()) {
            std::cmp::Ordering::Equal => self.0.cmp(&other.0),
            std::cmp::Ordering::Less => std::cmp::Ordering::Less,
            std::cmp::Ordering::Greater => std::cmp::Ordering::Greater,
        }
    }
}

impl PartialEq for LiteralEye {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl Eq for LiteralEye {}

impl std::fmt::Display for LiteralEye {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match &self.polarity() {
            true => write!(f, "{}", self.0),
            false => write!(f, "-{}", self.0),
        }
    }
}

impl std::ops::Not for LiteralEye {
    type Output = Self;

    fn not(self) -> Self::Output {
        self.negate()
    }
}
