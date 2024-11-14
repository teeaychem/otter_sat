use crate::db::keys::VariableIndex;

use super::{LiteralStruct, LiteralT};

impl LiteralT for LiteralStruct {
    fn negate(&self) -> Self {
        !*self
    }

    fn new(variable_id: VariableIndex, polarity: bool) -> Self {
        Self {
            v_id: variable_id,
            polarity,
        }
    }

    fn v_id(&self) -> VariableIndex {
        self.v_id
    }

    fn polarity(&self) -> bool {
        self.polarity
    }

    fn index(&self) -> usize {
        self.v_id as usize
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
        if self.v_id == other.v_id {
            if self.polarity == other.polarity {
                std::cmp::Ordering::Equal
            } else if self.polarity {
                std::cmp::Ordering::Greater
            } else {
                std::cmp::Ordering::Less
            }
        } else {
            self.v_id.cmp(&other.v_id)
        }
    }
}

impl PartialEq for LiteralStruct {
    fn eq(&self, other: &Self) -> bool {
        self.v_id == other.v_id && self.polarity == other.polarity
    }
}

impl Eq for LiteralStruct {}

impl std::fmt::Display for LiteralStruct {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self.polarity {
            true => write!(f, "{}", self.v_id),
            false => write!(f, "-{}", self.v_id),
        }
    }
}

impl std::ops::Not for LiteralStruct {
    type Output = Self;

    fn not(self) -> Self::Output {
        Self {
            v_id: self.v_id,
            polarity: !self.polarity,
        }
    }
}
