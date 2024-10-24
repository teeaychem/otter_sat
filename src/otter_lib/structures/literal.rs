use crate::structures::variable::VariableId;
use petgraph::graph::NodeIndex;

#[derive(Clone, Copy, Debug)]
pub struct Literal {
    v_id: VariableId,
    polarity: bool,
}

/// how a literal was settled
#[derive(Clone, Copy, Debug)]
pub enum Source {
    Choice,                // a choice made where the alternative may make a SAT difference
    HobsonChoice, // a choice made with a guarantee that the alternative would make no SAT difference
    Clause(NodeIndex), // the literal must be the case for SAT given some valuation
    Resolution(NodeIndex), // there was no reason to store the resolved clause
    Assumption,
}

impl Literal {
    pub fn negate(self) -> Self {
        !self
    }

    pub fn new(variable_id: VariableId, polarity: bool) -> Self {
        Self {
            v_id: variable_id,
            polarity,
        }
    }

    pub const fn v_id(self) -> VariableId {
        self.v_id
    }

    pub const fn polarity(self) -> bool {
        self.polarity
    }

    pub const fn index(self) -> usize {
        self.v_id as usize
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

impl std::ops::Not for Literal {
    type Output = Self;

    fn not(self) -> Self::Output {
        Literal {
            v_id: self.v_id,
            polarity: !self.polarity,
        }
    }
}
