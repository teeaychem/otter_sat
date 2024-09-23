use crate::structures::{StoredClause, VariableId};

use std::rc::Rc;

#[derive(Clone, Copy, Debug)]
pub struct Literal {
    pub v_id: VariableId,
    pub polarity: bool,
}

#[derive(Debug, PartialEq)]
pub enum LiteralError {
    NoVariable,
}

/// how a literal was settled
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LiteralSource {
    Choice,       // a choice made where the alternative may make a SAT difference
    HobsonChoice, // a choice made with a guarantee that the alternative would make no SAT difference
    Conflict,
    StoredClause(Rc<StoredClause>), // the literal must be the case for SAT given some valuation
    Assumption,
    Deduced
}

impl Literal {
    pub fn negate(&self) -> Self {
        Literal {
            v_id: self.v_id,
            polarity: !self.polarity,
        }
    }

    pub fn new(variable: VariableId, polarity: bool) -> Self {
        Literal {
            v_id: variable,
            polarity,
        }
    }
}

impl PartialOrd for Literal {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Literal {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        if self.v_id == other.v_id {
            if self.polarity == other.polarity {
                std::cmp::Ordering::Equal
            } else if self.polarity {
                std::cmp::Ordering::Less
            } else {
                std::cmp::Ordering::Greater
            }
        } else {
            self.v_id.cmp(&other.v_id)
        }
    }
}

impl PartialEq for Literal {
    fn eq(&self, other: &Self) -> bool {
        self.v_id == other.v_id && self.polarity == other.polarity
    }
}

impl Eq for Literal {}

impl std::fmt::Display for Literal {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self.polarity {
            true => write!(f, "{}", self.v_id),
            false => write!(f, "-{}", self.v_id),
        }
    }
}
