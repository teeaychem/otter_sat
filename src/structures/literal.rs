use crate::structures::{
    clause::stored_clause::StoredClause,
    variable::{Variable, VariableId},
};

use std::rc::Rc;

#[derive(Clone, Copy, Debug)]
pub struct Literal {
    pub v_id: VariableId,
    pub polarity: bool,
}

/// how a literal was settled
#[derive(Clone, Debug)]
pub enum LiteralSource {
    Choice,       // a choice made where the alternative may make a SAT difference
    HobsonChoice, // a choice made with a guarantee that the alternative would make no SAT difference
    StoredClause(Rc<StoredClause>), // the literal must be the case for SAT given some valuation
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
        Literal {
            v_id: variable,
            polarity,
        }
    }

    pub fn from_string(string: &str, vars: &mut Vec<Variable>) -> Literal {
        let trimmed_string = string.trim();

        if trimmed_string.is_empty() || trimmed_string == "-" {
            panic!("No variable when creating literal from string");
        }

        let polarity = !trimmed_string.starts_with('-');

        let mut the_name = trimmed_string;
        if !polarity {
            the_name = &the_name[1..]
        }

        let the_variable = {
            if let Some(variable) = vars.iter().find(|v| v.name() == the_name) {
                variable.id()
            } else {
                let the_id = vars.len() as VariableId;
                let new_variable = Variable::new(the_name, the_id);
                vars.push(new_variable);
                the_id
            }
        };
        let the_literal = Literal::new(the_variable, polarity);
        the_literal
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
