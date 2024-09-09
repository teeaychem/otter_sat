use crate::structures::{Clause, ClauseId, Formula, Level, Literal, Valuation, VariableId};
use petgraph::{
    algo::dominators::{self, simple_fast},
    prelude::NodeIndex,
    stable_graph::StableGraph,
    visit::NodeRef,
};
use std::collections::{BTreeSet, VecDeque};

// Implication graph

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ImplicationNode {
    literal: Literal,
    level: usize,
    conflict: bool,
}

#[derive(Clone, Debug)]
pub struct ImplicationEdge {
    clause_id: ClauseId,
}

/*
The graph will have at most as many nodes as variables, so a fixed size array can store where a variable appears in the graph.
 */
#[derive(Debug)]
pub struct ImplicationGraph {
    pub variable_indicies: Vec<Option<NodeIndex>>,
    pub conflict_indicies: Vec<NodeIndex>,
    pub graph: StableGraph<ImplicationNode, ImplicationEdge>,
}

impl ImplicationGraph {
    pub fn new_for(formula: &Formula) -> Self {
        ImplicationGraph {
            variable_indicies: vec![None; formula.var_count()],
            conflict_indicies: vec![],
            graph: StableGraph::new(),
        }
    }

    pub fn add_literal(&mut self, literal: Literal, level: usize, conflict: bool) -> NodeIndex {
        let index = self.graph.add_node(ImplicationNode {
            literal,
            level,
            conflict,
        });
        if !conflict {
            self.variable_indicies[literal.v_id] = Some(index);
        }
        index
    }

    pub fn get_or_make_literal(&mut self, literal: Literal, level: usize) -> NodeIndex {
        let literal_index = match self.variable_indicies[literal.v_id] {
            Some(index) => {
                let the_node = self.graph.node_weight(index).unwrap();
                if the_node.literal.polarity != literal.polarity {
                    // println!("Adding {} though {}", literal, the_node.literal);
                    panic!("Graph polarity does not match");
                }
                index
            }
            None => self.add_literal(literal, level, false),
        };
        literal_index
    }

    pub fn make_conflict(&mut self, literal: Literal, level: usize) -> NodeIndex {
        let index = self.add_literal(literal, level, true);
        self.conflict_indicies.push(index);
        index
    }

    pub fn get_literal(&self, literal: Literal) -> NodeIndex {
        let literal_index = match self.variable_indicies[literal.v_id] {
            Some(index) => {
                let the_node = self.graph.node_weight(index).unwrap();
                if the_node.literal.polarity != literal.polarity {
                    panic!("Graph polarity does not match");
                }
                index
            }
            None => {
                panic!("Unable to get literal")
            }
        };
        literal_index
    }

    pub fn add_choice(&mut self, literal: Literal, level: usize) -> NodeIndex {
        self.add_literal(literal, level, false)
    }

    pub fn add_implication(&mut self, clause: &Clause, consequent: Literal, level: usize) {
        println!(
            "Adding implication with consequent:\n{} : {}",
            clause, consequent
        );
        let consequent_index = self.get_or_make_literal(consequent, level);
        let antecedents = clause.literals.iter().filter(|l| l.v_id != consequent.v_id);
        for antecedent in antecedents {
            let antecedent_index = self.get_or_make_literal(antecedent.negate(), level);
            self.graph.add_edge(
                antecedent_index,
                consequent_index,
                ImplicationEdge {
                    clause_id: clause.id,
                },
            );
        }
    }

    pub fn add_conflict(&mut self, clause: &Clause, consequent: Literal, level: usize) {
        println!(
            "Adding implication with consequent:\n{} : {}",
            clause, consequent
        );
        let consequent_index = self.make_conflict(consequent, level);
        let antecedents = clause.literals.iter().filter(|l| l.v_id != consequent.v_id);
        for antecedent in antecedents {
            let antecedent_index = self.get_or_make_literal(antecedent.negate(), level);
            self.graph.add_edge(
                antecedent_index,
                consequent_index,
                ImplicationEdge {
                    clause_id: clause.id,
                },
            );
        }
    }

    pub fn remove_literal(&mut self, literal: Literal) {
        if let Some(index) = self.variable_indicies[literal.v_id] {
            self.graph.remove_node(index);
        };
        self.variable_indicies[literal.v_id] = None;
    }

    pub fn dominators(&self, index: NodeIndex) {
        println!("{:?}", self.graph);
        let x = simple_fast(&self.graph, index);
        let c_node = self.graph.node_weights().find(|w| w.conflict);
        println!(">>> {:?}", c_node);
        println!("{:?}", &x);
        if let Some(c) = c_node {
            let d = self.graph.node_indices().find(|x| self.graph.node_weight(*x).unwrap() == c).unwrap();
            println!("d ~ {:?}", d);
            let z = x.immediate_dominator(d);
            println!("Z {:?}", z);
        }
    }
}
