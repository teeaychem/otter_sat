use std::{collections::BTreeSet, fmt::Debug};

use crate::structures::{Literal, LiteralSource, Solve, Variable, VariableId};

use super::{literal, ClauseId};

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
    implications: ImplicationGraph,
}

impl Level {
    fn new(index: usize, assignment: &Assignment) -> Self {
        Level {
            index,
            choices: vec![],
            observations: vec![],
            implications: ImplicationGraph::new(assignment),
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

pub type EdgeId = usize;
pub type ImplicationGraphEdge = (VariableId, ClauseId, VariableId);

#[derive(Clone, Debug)]
pub struct ImplicationGraph {
    backwards: Vec<Option<BTreeSet<EdgeId>>>, // indicies correspond to variables, indexed vec is for edges
    edges: Vec<ImplicationGraphEdge>,
}

impl ImplicationGraph {
    pub fn new(assignment: &Assignment) -> Self {
        ImplicationGraph {
            backwards: vec![None; assignment.valuation.status.len()],
            edges: vec![],
        }
    }

    pub fn from(assignment: &Assignment, solve: &Solve) -> ImplicationGraph {
        let the_units = solve.find_all_units_on(&assignment.valuation, &mut BTreeSet::new());
        let edges = the_units
            .iter()
            .flat_map(|(clause_id, to_literal)| {
                solve.clauses[*clause_id as usize]
                    .literals
                    .iter()
                    .filter(|&l| l != to_literal)
                    .map(|from_literal| (from_literal.v_id, *clause_id, to_literal.v_id))
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();
        let mut the_graph = ImplicationGraph {
            backwards: vec![None; assignment.valuation.status.len()],
            edges,
        };
        the_graph.add_backwards();
        the_graph
    }

    pub fn add_backwards(&mut self) {
        for (edge_id, (_from_node, _clause_id, to_node)) in self.edges.iter().enumerate() {
            if let Some(Some(set)) = self.backwards.get_mut(*to_node as usize) {
                set.insert(edge_id);
            } else {
                self.backwards[*to_node as usize] = Some(BTreeSet::from([edge_id]));
            };
        }
    }
}

impl Assignment {
    pub fn fresh_level(&mut self) -> &mut Level {
        let level_cout = self.levels.len();
        let the_level = Level::new(self.levels.len(), self);
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

    pub fn make_implication_for_last_level(&mut self, solve: &Solve) {
        let the_graph = ImplicationGraph::from(self, solve);
        println!("the graph: {:?}", the_graph);
        self.last_level_mut().implications = the_graph;
    }
}

#[derive(Debug, Clone)]
pub struct Valuation {
    status: Vec<Option<bool>>,
}

impl Valuation {
    pub fn maybe_clear_level(&mut self, maybe_level: &Option<Level>) {
        if let Some(level) = maybe_level {
            for literal in level.literals() {
                self.status[literal.v_id as usize] = None;
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
        let mut the_assignment = Assignment {
            valuation: Valuation::new(solve.vars().len()),
            levels: vec![],
        };
        let level_zero = Level::new(0, &the_assignment);
        the_assignment.levels.push(level_zero);
        the_assignment
    }

    // the last choice corresponds to the curent depth
    pub fn pop_last_level(&mut self) -> Option<Level> {
        if self.levels.len() <= 1 {
            return None;
        }
        let the_level = self.levels.pop();
        self.valuation.maybe_clear_level(&the_level);
        the_level
    }

    pub fn set(&mut self, literal: &Literal, source: LiteralSource) {
        self.valuation.set(literal);
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

    pub fn set(&mut self, literal: &Literal) {
        self.status[literal.v_id as usize] = Some(literal.polarity)
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
