use crate::structures::{Clause, ClauseId, Formula, Level, Literal, Valuation, VariableId};
use petgraph::{
    algo::dominators::{self, simple_fast},
    dot::{Config, Dot},
    graph::edge_index,
    prelude::NodeIndex,
    stable_graph::StableGraph,
    visit::NodeRef,
};
use std::collections::{BTreeSet, VecDeque};

macro_rules! target_graph {
    () => {
        "graph"
    };
}

// Implication graph

#[derive(Clone, Debug)]
pub struct ImplicationNode {
    pub conflict: bool,
    pub level: usize,
    pub literal: Literal,
}

#[derive(Clone, Copy, Debug)]
enum Source {
    Clause(ClauseId),
    Contradiction,
}

#[derive(Clone, Debug)]
pub struct ImplicationEdge {
    source: Source,
}

/*
The graph will have at most as many nodes as variables, so a fixed size array can store where a variable appears in the graph.
 */
#[derive(Debug)]
pub struct ImplicationGraph {
    variable_indicies: Vec<Option<NodeIndex>>,
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
                    panic!("Graph polarity does not match");
                }
                if the_node.literal.v_id != literal.v_id {
                    panic!("Variables do not match!");
                }
                if the_node.level != level {
                    panic!("Levels do not match!");
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
                panic!("Unable to get {}", literal)
            }
        };
        literal_index
    }

    pub fn add_choice(&mut self, literal: Literal, level: usize) -> NodeIndex {
        let choice_index = self.add_literal(literal, level, false);
        log::trace!(target: target_graph!(), "+Choice @{level}: {literal}");
        choice_index
    }

    pub fn add_implication(&mut self, clause: &Clause, to: Literal, level: usize, conflict: bool) {
        let (consequent_index, description) = if conflict {
            (self.make_conflict(to, level), "Conflict")
        } else {
            (self.get_or_make_literal(to, level), "Implication")
        };
        let antecedents = clause.literals.iter().filter(|a| a.v_id != to.v_id);
        for antecedent in antecedents {
            let antecedent_index = self.get_literal(antecedent.negate());
            let edge_index = self.graph.add_edge(
                antecedent_index,
                consequent_index,
                ImplicationEdge {
                    source: Source::Clause(clause.id),
                },
            );
            log::debug!(target: target_graph!(), "+{description} @{level}: {antecedent} --[{}]-> {to}", edge_index.index());
        }
        log::info!(target: target_graph!(), "+{description} @{level}: {clause} -> {to}");
    }

    pub fn add_contradiction(&mut self, from: Literal, to: Literal, level: usize) {
        let choice_index = self.get_literal(from);
        let contradiction_index = self.get_or_make_literal(to, level);

        let edge_index = self.graph.add_edge(
            choice_index,
            contradiction_index,
            ImplicationEdge {
                source: Source::Contradiction,
            },
        );
        log::debug!(target: target_graph!(), "+Contradiction @{level} {from} --[{}]-> {to}", edge_index.index());
    }

    pub fn remove_literal(&mut self, literal: Literal) {
        if let Some(index) = self.variable_indicies[literal.v_id] {
            let node = self.graph.remove_node(index);
            log::debug!(target: target_graph!(), "-Removed: {node:?}");
            self.variable_indicies[literal.v_id] = None;
        };
    }

    pub fn remove_conflicts(&mut self) {
        for conflict in &self.conflict_indicies {
            self.graph.remove_node(*conflict);
        }
        self.conflict_indicies.clear();
    }

    pub fn dominators(&self, root: NodeIndex, conflict: NodeIndex) {
        let x = simple_fast(&self.graph, root);

        let z = x.immediate_dominator(conflict);
        println!("the immediate dominator of {:?} is {:?}", root, z);
        println!(
            "the dominators of {:?} are {:?}",
            root,
            x.dominators(conflict)
        );
        if z.is_none() {
            println!(
                "{:?}",
                Dot::with_config(&self.graph, &[Config::EdgeIndexLabel])
            );
            panic!("No dominator")
        }
    }
}

impl<'borrow> ImplicationGraph {
    pub fn remove_literals<I>(&'borrow mut self, literals: I)
    where
        I: Iterator<Item = Literal>,
    {
        for literal in literals {
            self.remove_literal(literal)
        }
    }
}
