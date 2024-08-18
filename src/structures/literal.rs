use crate::structures::ClauseId;

pub type VariableId = u32;

#[derive(Debug)]
pub struct Variable {
    pub name: String,
    pub id: VariableId,
}

#[derive(Clone, Debug)]
pub struct Literal {
    variable: VariableId,
    polarity: bool,
}

#[derive(Debug)]
pub enum LiteralError {
    NoVariable,
}

impl std::fmt::Display for Literal {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self.polarity {
            true => write!(f, "{}", self.variable),
            false => write!(f, "-{}", self.variable),
        }
    }
}

/// how a literal was added to an assignment
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LiteralSource {
    Choice,     // a choice made where the alternative may make a SAT difference
    FreeChoice, // a choice made with a guarantee that the alternative would make no SAT difference
    DeductionFalsum,
    DeductionClause(ClauseId), // the literal must be the case for SAT given some assignment
    Assumption,
}

impl Literal {
    pub fn negate(&self) -> Self {
        Literal {
            variable: self.variable,
            polarity: !self.polarity,
        }
    }

    pub fn new(variable: VariableId, polarity: bool) -> Self {
        Literal { variable, polarity }
    }

    pub fn v_id(&self) -> VariableId {
        self.variable
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
