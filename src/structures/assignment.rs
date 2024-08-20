use std::fmt::Debug;

use crate::structures::{Literal, LiteralSource, Solve, Variable, VariableId};

use super::ClauseId;

/// a partial assignment with some history
#[derive(Clone, Debug)]
pub struct Assignment {
    pub valuation: Valuation,
    levels: Vec<Level>,
}

#[derive(Clone, Debug)]
pub struct Level {
    index: usize,
    pub choices: Vec<Literal>,
    observations: Vec<Literal>,
    implications: Vec<(Literal, ClauseId, Literal)>,
}

impl Level {
    fn new(index: usize) -> Self {
        Level {
            index,
            choices: vec![],
            observations: vec![],
            implications: vec![],
        }
    }

    pub fn add_literal(&mut self, literal: &Literal, source: LiteralSource) {
        match source {
            LiteralSource::Choice => self.choices.push(literal.clone()),
            _ => todo!(),
        }
    }

    pub fn literals(&self) -> Vec<&Literal> {
        self.choices
            .iter()
            .chain(self.observations.iter())
            .collect()
    }
}

impl Assignment {
    pub fn fresh_level(&mut self) -> &mut Level {
        let level_cout = self.levels.len();
        let the_level = Level::new(self.levels.len());
        self.levels.push(the_level);
        &mut self.levels[level_cout]
    }

    pub fn last_level(&self) -> &Level {
        self.levels.last().unwrap()
    }

    pub fn level_mut(&mut self, index: usize) -> &mut Level {
        &mut self.levels[index]
    }

    pub fn last_level_mut(&mut self) -> &mut Level {
        let last_position = self.levels.len() - 1;
        self.level_mut(last_position)
    }
}

#[derive(Clone, Debug)]
pub struct Record {
    depth: usize,
    literal: Literal,
    source: LiteralSource,
}

#[derive(Debug, Clone)]
pub struct Valuation {
    status: Vec<Option<bool>>,
}

impl Valuation {
    pub fn maybe_clear_level(&mut self, maybe_level: &Option<Level>) {
        if let Some(level) = maybe_level {
            for literal in level.literals() {
                self.status[literal.v_id() as usize] = None;
            }
        }
    }
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

impl Assignment {
    pub fn for_solve(solve: &Solve) -> Self {
        Assignment {
            valuation: Valuation::new(solve.vars().len()),
            levels: vec![Level::new(0)],
        }
    }

    // the last choice corresponds to the curent depth
    pub fn pop_last_level(&mut self) -> Option<Level> {
        if self.levels.is_empty() {
            return None;
        }
        let the_level = self.levels.pop();
        self.valuation.maybe_clear_level(&the_level);
        the_level
    }

    pub fn set(&mut self, literal: &Literal, source: LiteralSource) {
        self.valuation.set(literal.clone());
        match source {
            LiteralSource::Choice => {
                let fresh_level = self.fresh_level();
                fresh_level.add_literal(literal, source);
            }
            LiteralSource::HobsonChoice | LiteralSource::Assumption => {
                self.level_mut(0).observations.push(literal.clone());
            }
            LiteralSource::DeductionClause(_) | LiteralSource::Conflict => {
                self.last_level_mut().observations.push(literal.clone());
            }
        };
    }

    pub fn get_unassigned_id(&self, solve: &Solve) -> Option<VariableId> {
        solve
            .vars()
            .iter()
            .find(|&v| self.valuation.get_by_variable(v).is_ok_and(|p| p.is_none()))
            .map(|found| found.id)
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
