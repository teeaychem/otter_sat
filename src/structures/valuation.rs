use crate::structures::{Level, Literal, Solve, SolveError, VariableId};
// Valuation

pub type ValuationVec = Vec<Option<bool>>;

pub trait Valuation {
    fn new_for_variables(variable_count: usize) -> Self;

    fn as_display_string(&self, solve: &Solve) -> String;

    fn of_v_id(&self, v_id: VariableId) -> Result<Option<bool>, SolveError>;

    fn set_literal(&mut self, literal: &Literal) -> Result<(), ValuationError>;

    fn clear_v_id(&mut self, v_id: VariableId);

    fn clear_if_level(&mut self, maybe_level: &Option<Level>);

    fn size(&self) -> usize;

    fn literals(&self) -> Vec<Literal>;
}

pub enum ValuationError {
    Inconsistent,
}

impl Valuation for ValuationVec {
    fn new_for_variables(variable_count: usize) -> Self {
        vec![None; variable_count + 1]
    }

    fn as_display_string(&self, solve: &Solve) -> String {
        self.iter()
            .enumerate()
            .filter(|(_, p)| p.is_some())
            .map(|(i, p)| {
                let variable = solve.formula.var_by_id(i as VariableId).unwrap();
                match p {
                    Some(true) => variable.name.to_string(),
                    Some(false) => format!("-{}", variable.name),
                    _ => String::new(),
                }
            })
            .collect::<Vec<_>>()
            .join(" ")
    }

    fn of_v_id(&self, v_id: VariableId) -> Result<Option<bool>, SolveError> {
        if let Some(&info) = self.get(v_id as usize) {
            Ok(info)
        } else {
            Err(SolveError::OutOfBounds)
        }
    }

    fn set_literal(&mut self, literal: &Literal) -> Result<(), ValuationError> {
        if let Some(already_set) = self[literal.v_id as usize] {
            if already_set == literal.polarity {
                Ok(())
            } else {
                Err(ValuationError::Inconsistent)
            }
        } else {
            self[literal.v_id as usize] = Some(literal.polarity);
            Ok(())
        }
    }

    fn clear_v_id(&mut self, v_id: VariableId) {
        self[v_id as usize] = None
    }

    fn clear_if_level(&mut self, maybe_level: &Option<Level>) {
        if let Some(level) = maybe_level {
            level
                .literals()
                .for_each(|l| self.clear_v_id(l.v_id));
        }
    }

    fn size(&self) -> usize {
        self.len()
    }

    fn literals(&self) -> Vec<Literal> {
        self.iter()
            .enumerate()
            .filter(|(_, v)| v.is_some())
            .map(|(i, v)| Literal::new(i.try_into().unwrap(), v.unwrap()))
            .collect::<Vec<_>>()
    }
}
