use crate::structures::{
    Clause, ClauseId, Formula, Level, Literal, StoredClause, Valuation, VariableId,
};
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
pub enum ImplicationSource {
    StoredClause(ClauseId),
    Contradiction,
}

#[derive(Clone, Debug)]
pub struct ImplicationEdge {
    source: ImplicationSource,
}

/*
The graph will have at most as many nodes as variables, so a fixed size array can store where a variable appears in the graph.
 */
#[derive(Debug)]
pub struct ImplicationGraph {
    variable_indicies: Vec<Option<NodeIndex>>,
    pub graph: StableGraph<ImplicationNode, ImplicationEdge>,
}

impl ImplicationGraph {
    pub fn new_for(formula: &Formula) -> Self {
        ImplicationGraph {
            variable_indicies: vec![None; formula.var_count()],
            graph: StableGraph::new(),
        }
    }

    pub fn add_literal(&mut self, literal: Literal, level: usize, conflict: bool) -> NodeIndex {
        log::trace!(target: target_graph!(), "?+ Literal @{level}: {literal}");
        let index = self.graph.add_node(ImplicationNode {
            literal,
            level,
            conflict,
        });
        if !conflict {
            self.variable_indicies[literal.v_id] = Some(index);
        }
        log::trace!(target: target_graph!(), "+Literal @{level}: {literal}");
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
                println!(
                    "{:?}",
                    Dot::with_config(&self.graph, &[Config::EdgeIndexLabel])
                );
                panic!("Unable to get {}", literal)
            }
        };
        literal_index
    }

    pub fn add_implication(
        &mut self,
        clause: &impl Clause,
        to: Literal,
        level: usize,
        conflict: bool,
        source: ImplicationSource
    ) -> NodeIndex {
        log::warn!(target: target_graph!(), "+Implication {} -> {to}", clause.as_string());
        let (consequent_index, description) = if conflict {
            (self.add_literal(to, level, true), "Conflict")
        } else {
            (self.get_or_make_literal(to, level), "Implication")
        };
        for antecedent in clause.literals().filter(|a| a.v_id != to.v_id) {
            let edge_index = self.graph.add_edge(
                self.get_literal(antecedent.negate()),
                consequent_index,
                ImplicationEdge {
                    source,
                },
            );
            log::debug!(target: target_graph!(), "+{description} @{level}: {antecedent} --[{}]-> {to}", edge_index.index());
        }
        log::info!(target: target_graph!(), "+{description} @{level}: {} -> {to}", clause.as_string());
        consequent_index
    }

    pub fn add_contradiction(&mut self, from: Literal, to: Literal, level: usize) {
        let choice_index = self.get_literal(from);
        let contradiction_index = self.get_or_make_literal(to, level);

        let edge_index = self.graph.add_edge(
            choice_index,
            contradiction_index,
            ImplicationEdge {
                source: ImplicationSource::Contradiction,
            },
        );
        log::debug!(target: target_graph!(), "+Contradiction @{level} {from} --[{}]-> {to}", edge_index.index());
    }

    pub fn remove_literal(&mut self, literal: Literal) {
        if let Some(index) = self.variable_indicies[literal.v_id] {
            if let Some(node) = self.graph.remove_node(index) {
                log::debug!(target: target_graph!(), "-{node:?}");
                self.variable_indicies[literal.v_id] = None;
            } else {
                panic!("Failed to remove node")
            }
        };
    }

    pub fn remove_node(&mut self, index: NodeIndex) {
        if let Some(node) = self.graph.remove_node(index) {
            log::debug!(target: target_graph!(), "-{node:?}");
        } else {
            panic!("Failed to remove node")
        }
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
            println!("\nNo dominator\n");
            log::warn!("No dominator")
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

    pub fn remove_level(&mut self, level: &Level) {
        self.remove_literals(level.literals());
        if self.graph.node_weights().filter(|n| n.conflict).count() != 0 {
            panic!("Removing a level while conflicts exist");
        };
    }
}
