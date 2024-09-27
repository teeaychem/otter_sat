use std::collections::binary_heap::Iter;

use crate::structures::{solve::Solve, Level, Literal, VariableId};
// Valuation

pub type ValuationVec = Vec<Option<bool>>;

pub trait Valuation {
    fn new_for_variables(variable_count: usize) -> Self;

    fn as_display_string(&self, solve: &Solve) -> String;

    fn as_internal_string(&self) -> String;

    fn of_v_id(&self, v_id: VariableId) -> Option<bool>;

    fn check_literal(&self, literal: Literal) -> Result<ValuationOk, ValuationError>;

    fn update_value(&mut self, literal: Literal) -> Result<(), ValuationError>;

    fn clear_v_id(&mut self, v_id: VariableId);

    fn clear_level(&mut self, level: &Level);

    fn some_none(&self) -> Option<VariableId>;

    fn to_vec(&self) -> ValuationVec;

    fn values(&self) -> impl Iterator<Item = Option<bool>>;
}

pub enum ValuationError {
    Match,
    Conflict,
}

pub enum ValuationOk {
    NotSet,
    Match,
}

impl Valuation for ValuationVec {
    fn new_for_variables(variable_count: usize) -> Self {
        vec![None; variable_count]
    }

    fn as_display_string(&self, solve: &Solve) -> String {
        self.iter()
            .enumerate()
            .filter(|(_, p)| p.is_some())
            .map(|(i, p)| {
                let variable = solve.var_by_id(i as VariableId).unwrap();
                match p {
                    Some(true) => variable.name().to_string(),
                    Some(false) => format!("-{}", variable.name()),
                    _ => String::new(),
                }
            })
            .collect::<Vec<_>>()
            .join(" ")
    }

    fn as_internal_string(&self) -> String {
        self.iter()
            .enumerate()
            .filter(|(_, p)| p.is_some())
            .map(|(i, p)| match p {
                Some(true) => format!("{}", i),
                Some(false) => format!("-{}", i),
                _ => String::new(),
            })
            .collect::<Vec<_>>()
            .join(" ")
    }

    fn of_v_id(&self, v_id: VariableId) -> Option<bool> {
        match self.get(v_id) {
            Some(&info) => info,
            None => panic!("Read of variable outside of valuation"),
        }
    }

    fn check_literal(&self, literal: Literal) -> Result<ValuationOk, ValuationError> {
        match self[literal.v_id] {
            Some(already_set) if already_set == literal.polarity => Ok(ValuationOk::Match),
            Some(_already_set) => Err(ValuationError::Conflict),
            None => Ok(ValuationOk::NotSet),
        }
    }

    fn update_value(&mut self, literal: Literal) -> Result<(), ValuationError> {
        match self[literal.v_id] {
            Some(value) if value != literal.polarity => Err(ValuationError::Conflict),
            Some(_value) => Err(ValuationError::Match),
            None => {
                self[literal.v_id] = Some(literal.polarity);
                Ok(())
            }
        }
    }

    fn clear_v_id(&mut self, v_id: VariableId) {
        self[v_id] = None
    }

    fn clear_level(&mut self, level: &Level) {
        for literal in level.literals() {
            self.clear_v_id(literal.v_id);
        }

        level.literals().for_each(|l| self.clear_v_id(l.v_id));
    }

    fn some_none(&self) -> Option<VariableId> {
        self.iter()
            .enumerate()
            .filter(|(_, val)| val.is_none())
            .map(|(i, _)| i)
            .next()
        // .last()
    }

    fn to_vec(&self) -> ValuationVec {
        self.clone()
    }

    fn values(&self) -> impl Iterator<Item = Option<bool>> {
        self.iter().cloned()
    }
}
