use std::fmt::Debug;

use crate::structures::{Literal, Solve, Variable, VariableId};

#[derive(Debug, Clone)]
pub struct Valuation {
    status: Vec<Option<bool>>,
}

#[derive(PartialEq)]
pub enum AssignmentError {
    OutOfBounds,
}

impl std::fmt::Display for Valuation {
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

impl Valuation {
    pub fn new(variable_count: usize) -> Self {
        Valuation {
            status: vec![None; variable_count + 1],
        }
    }

    pub fn as_external_string(&self, solve: &Solve) -> String {
        self.status
            .iter()
            .enumerate()
            .filter(|(_, p)| p.is_some())
            .map(|(i, p)| {
                let variable = solve.var_by_id(i as VariableId).unwrap();
                match p {
                    Some(true) => variable.name.to_string(),
                    Some(false) => format!("-{}", variable.name),
                    _ => String::new(),
                }
            })
            .collect::<Vec<_>>()
            .join(" ")
    }

    pub fn get_by_variable(&self, variable: &Variable) -> Result<Option<bool>, AssignmentError> {
        self.get_by_variable_id(variable.id)
    }

    pub fn get_by_variable_id(&self, v_id: VariableId) -> Result<Option<bool>, AssignmentError> {
        if let Some(&info) = self.status.get(v_id as usize) {
            Ok(info)
        } else {
            Err(AssignmentError::OutOfBounds)
        }
    }

    pub fn set(&mut self, literal: Literal) {
        self.status[literal.v_id() as usize] = Some(literal.polarity())
    }

    pub fn clear(&mut self, v_id: VariableId) {
        self.status[v_id as usize] = None
    }

    pub fn get_unassigned(&self) -> Option<VariableId> {
        if let Some((index, _)) = self
            .status
            .iter()
            .enumerate()
            .find(|(i, v)| *i > 0 && v.is_none())
        {
            Some(index as VariableId)
        } else {
            None
        }
    }
}
