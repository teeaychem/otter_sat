use crate::structures::{Literal, Variable, VariableId};

#[derive(Debug)]
pub struct Assignment {
    status: Vec<Option<bool>>,
}

#[derive(PartialEq)]
pub enum AssignmentError {
    OutOfBounds
}

impl std::fmt::Display for Assignment {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "[")?;
        for maybe_literal in self.status.iter() {
            if let Some(literal) = maybe_literal {
                write!(f, "{}", literal)?
            } else {
                write!(f, " ❍ ")?
            }
        }
        write!(f, "]")
    }
}

impl Assignment {
    pub fn new(variable_count: usize) -> Self {
        Assignment {
            status: vec![None; variable_count + 1],
        }
    }

    pub fn get(&self, variable: &Variable) -> Result<Option<bool>, AssignmentError> {
        if let Some(&info) = self.status.get(variable.id as usize) {
            Ok(info)
        } else {
            Err(AssignmentError::OutOfBounds)
        }
    }

    pub fn set(&mut self, literal: Literal) {
        println!("settings: {:?}", literal);
        self.status[literal.variable().id as usize] = Some(literal.polarity())
    }

    pub fn clear(&mut self, index: &Variable) {
        self.status[index.id as usize] = None
    }

    pub fn get_unassigned(&self) -> Option<VariableId> {
        if let Some((index, _)) = self.status.iter().enumerate().find(|(i, v)| *i > 0 && v.is_none()) {
            Some(index as VariableId)
        } else {
            None
        }
    }
}
