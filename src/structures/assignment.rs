use std::{collections::BTreeSet, fmt::Debug};

use crate::structures::{
    ClauseId, Literal, LiteralSource, Solve, Valuation, ValuationVec, VariableId,
};

/// a partial assignment with some history
#[derive(Clone, Debug)]
pub struct Assignment {
    pub valuation: Vec<Option<bool>>,
    pub levels: Vec<Level>,
}

#[derive(PartialEq)]
pub enum AssignmentError {
    OutOfBounds,
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

impl Assignment {
    pub fn fresh_level(&mut self) -> &mut Level {
        let level_cout = self.levels.len();
        let the_level = Level::new(self.levels.len(), self);
        self.levels.push(the_level);
        &mut self.levels[level_cout]
    }

    pub fn current_level(&self) -> usize {
        self.levels.len() - 1
    }

    // pub fn last_level(&self) -> &Level {
    //     self.levels.last().unwrap()
    // }

    pub fn level_mut(&mut self, index: usize) -> &mut Level {
        &mut self.levels[index]
    }

    pub fn last_level_mut(&mut self) -> &mut Level {
        let last_position = self.levels.len() - 1;
        self.level_mut(last_position)
    }

    // pub fn level_from_choice(&mut self, choice: &Literal, solve: &Solve) {
    //     let the_level = self.fresh_level();
    //     the_level.choices.push(choice.clone());
    //     // let the_graph = ImplicationGraph::for_level(&self.valuation, solve);
    //     // println!("the graph: {:?}", the_graph);
    //     // self.last_level_mut().implications = the_graph;
    // }

    pub fn for_solve(solve: &Solve) -> Self {
        let mut the_assignment = Assignment {
            valuation: Vec::<Option<bool>>::new_for_variables(solve.vars().len()),
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
        self.valuation.clear_if_level(&the_level);
        the_level
    }

    pub fn set(&mut self, literal: &Literal, source: LiteralSource) {
        self.valuation.set_literal(literal);
        match source {
            LiteralSource::Choice => {
                let fresh_level = self.fresh_level();
                fresh_level.add_literal(literal, source);
            }
            LiteralSource::HobsonChoice | LiteralSource::Assumption => {
                self.level_mut(0).observations.push(literal.clone());
            }
            LiteralSource::Clause(_) | LiteralSource::Conflict => {
                self.last_level_mut().observations.push(literal.clone());
            }
        };
    }

    pub fn get_unassigned_id(&self, solve: &Solve) -> Option<VariableId> {
        solve
            .vars()
            .iter()
            .find(|&v| self.valuation.of_v_id(v.id).is_ok_and(|p| p.is_none()))
            .map(|found| found.id)
    }

    pub fn valuation_at_level(&self, index: usize) -> ValuationVec {
        let mut valuation = ValuationVec::new_for_variables(self.valuation.len());
        (0..=index).for_each(|i| {
            self.levels[i]
                .literals()
                .iter()
                .for_each(|l| valuation.set_literal(l))
        });
        valuation
    }

    pub fn add_implication_graph_for_level(&mut self, index: usize, solve: &Solve) {
        // let valuation = self.valuation_at_level(index);
        let the_graph = ImplicationGraph::for_level(self, index, solve);
        self.levels[index].implications = the_graph;
    }

    pub fn graph_at_level(&self, index: usize) -> &ImplicationGraph {
        &self.levels[index].implications
    }

    pub fn add_literals_from_graph(&mut self, index: usize) {
        let the_level = &mut self.levels[index];
        the_level.implications.units.iter().for_each(|l| {
            self.valuation.set_literal(l);
            the_level.observations.push(l.clone());
        })
    }
}

// Implication graph

pub type EdgeId = usize;
pub type ImplicationGraphEdge = (VariableId, ClauseId, VariableId);

#[derive(Clone, Debug)]
pub struct ImplicationGraph {
    pub units: Vec<Literal>,
    backwards: Vec<Option<BTreeSet<EdgeId>>>, // indicies correspond to variables, indexed vec is for edges
    edges: Vec<ImplicationGraphEdge>,
}

impl ImplicationGraph {
    pub fn new(assignment: &Assignment) -> Self {
        ImplicationGraph {
            units: vec![],
            backwards: vec![None; assignment.valuation.len()],
            edges: vec![],
        }
    }

    pub fn for_level(assignment: &Assignment, level: usize, solve: &Solve) -> ImplicationGraph {
        let valuation = &assignment.valuation_at_level(level);
        let the_units = solve.find_all_units_on(valuation, &mut BTreeSet::new());
        let units: Vec<Literal> = the_units
            .iter()
            .map(|(_clause, literal)| literal)
            .cloned()
            .collect();

        let relevant_ids = units
            .iter()
            .chain(assignment.levels[level].literals().iter().cloned())
            .map(|l| l.v_id)
            .collect::<BTreeSet<_>>();

        let mut relevant_edges: Vec<ImplicationGraphEdge> = vec![];
        for (clause_id, to_literal) in the_units {
            for from_literal in &solve.clauses[clause_id].literals {
                if relevant_ids.contains(&from_literal.v_id) && *from_literal != to_literal {
                    relevant_edges.push((from_literal.v_id, clause_id, to_literal.v_id));
                }
            }
        }

        // let edges = the_units
        //     .iter()
        //     .flat_map(|(clause_id, to_literal)| {
        //         solve.clauses[*clause_id]
        //             .literals
        //             .iter()
        //             .filter(|&l| l != to_literal)
        //             .map(|from_literal| (from_literal.v_id, *clause_id, to_literal.v_id))
        //             .collect::<Vec<_>>()
        //     })
        //     .collect::<Vec<_>>();
        let mut the_graph = ImplicationGraph {
            units,
            backwards: vec![None; valuation.size()],
            edges: relevant_edges,
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
