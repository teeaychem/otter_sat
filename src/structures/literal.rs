use crate::structures::ClauseId;

pub type VariableId = u32;

#[derive(Debug)]
pub struct Variable {
    pub name: String,
    pub id: VariableId,
}

#[derive(Clone, Debug)]
pub struct Literal {
    pub v_id: VariableId,
    pub polarity: bool,
}

#[derive(Debug)]
pub enum LiteralError {
    NoVariable,
}

impl std::fmt::Display for Literal {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self.polarity {
            true => write!(f, "{}", self.v_id),
            false => write!(f, "-{}", self.v_id),
        }
    }
}

/// how a literal was added to an assignment
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LiteralSource {
    Choice,     // a choice made where the alternative may make a SAT difference
    HobsonChoice, // a choice made with a guarantee that the alternative would make no SAT difference
    Conflict,
    Clause(ClauseId), // the literal must be the case for SAT given some assignment
    Assumption,
}

impl Literal {
    pub fn negate(&self) -> Self {
        Literal {
            v_id: self.v_id,
            polarity: !self.polarity,
        }
    }

    pub fn new(variable: VariableId, polarity: bool) -> Self {
        Literal { v_id: variable, polarity }
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
