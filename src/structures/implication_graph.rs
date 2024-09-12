use crate::{
    binary_resolution,
    clause::ClauseVec,
    structures::{
        Clause, ClauseId, Formula, Level, Literal, Solve, StoredClause, Valuation, VariableId,
    },
};
use petgraph::{
    algo::{
        dominators::{self, simple_fast},
        simple_paths,
    },
    dot::{Config, Dot},
    graph::edge_index,
    prelude::NodeIndex,
    stable_graph::StableGraph,
    visit::NodeRef,
};
use std::collections::BTreeSet;

macro_rules! target_graph {
    () => {
        "graph"
    };
}

// Implication graph

#[derive(Clone, Debug)]
enum NodeItem {
    Literal(Literal),
    Falsum,
}

#[derive(Clone, Debug)]
struct ImplicationNode {
    level: usize,
    item: NodeItem,
}

#[derive(Clone, Copy, Debug)]
pub enum ImplicationSource {
    StoredClause(ClauseId),
    Contradiction,
    Conflict,
}

#[derive(Clone, Debug)]
pub struct ImplicationEdge {
    pub source: ImplicationSource,
}

/*
The graph will have at most as many nodes as variables, so a fixed size array can store where a variable appears in the graph.
 */
#[derive(Debug)]
pub struct ImplicationGraph {
    variable_indicies: Vec<Option<NodeIndex>>,
    graph: StableGraph<ImplicationNode, ImplicationEdge>,
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
            item: NodeItem::Literal(literal),
            level,
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
                if the_node.level != level {
                    panic!("Levels do not match!");
                }
                match the_node.item {
                    NodeItem::Falsum => panic!("get falsum"),
                    NodeItem::Literal(l) => {
                        if l.polarity != literal.polarity {
                            panic!("Graph polarity does not match");
                        }
                        if l.v_id != literal.v_id {
                            panic!("Variables do not match!");
                        }
                        index
                    }
                }
            }
            None => self.add_literal(literal, level, false),
        };
        literal_index
    }

    pub fn get_literal(&self, literal: Literal) -> NodeIndex {
        let literal_index = match self.variable_indicies[literal.v_id] {
            Some(index) => {
                let the_node = self.graph.node_weight(index).unwrap();
                match the_node.item {
                    NodeItem::Falsum => panic!("Get falsum"),
                    NodeItem::Literal(node_literal) => {
                        if node_literal.polarity != literal.polarity {
                            panic!("Graph polarity does not match");
                        }
                        index
                    }
                }
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
        clause: impl Iterator<Item = Literal>,
        to: Literal,
        level: usize,
        source: ImplicationSource,
    ) -> NodeIndex {
        // log::warn!(target: target_graph!(), "+Implication {} -> {to}", clause.as_string());
        let (consequent_index, description) = (self.get_or_make_literal(to, level), "Implication");
        for antecedent in clause.filter(|a| a.v_id != to.v_id) {
            let edge_index = self.graph.add_edge(
                self.get_literal(antecedent),
                consequent_index,
                ImplicationEdge { source },
            );
            let the_edge = self.graph.edge_weight(edge_index).unwrap();
            match the_edge.source {
                ImplicationSource::StoredClause(c) => {
                    log::debug!(target: target_graph!(), "+{description} @{level}: {antecedent} --[{c}]-> {to}")
                }
                _ => {
                    log::debug!(target: target_graph!(), "+{description} @{level}: {antecedent} --[{the_edge:?}]-> {to}")
                }
            }
        }
        // log::info!(target: target_graph!(), "+{description} @{level}: {} -> {to}", clause.as_string());
        consequent_index
    }

    pub fn add_temporary_falsum(&mut self, clause: &impl Clause) -> NodeIndex {
        let falsum = self.graph.add_node(ImplicationNode {
            level: 0, // as the falsum is temporary and the level is unimportant, it's fixed to 0
            item: NodeItem::Falsum,
        });
        for antecedent in clause.literals() {
            let _edge_index = self.graph.add_edge(
                self.get_literal(antecedent.negate()),
                falsum,
                ImplicationEdge {
                    source: ImplicationSource::Conflict,
                },
            );
        }
        falsum
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
        log::debug!(target: target_graph!(), "+Contradiction @{level} {from} --[{:?}]-> {to}", self.graph.edge_weight(edge_index));
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

    pub fn immediate_dominator(&self, root: NodeIndex, conflict: NodeIndex) -> Option<Literal> {
        let dominators = simple_fast(&self.graph, root);

        let i_d_index = dominators.immediate_dominator(conflict);
        println!("the immediate dominator of {:?} is {:?}", root, i_d_index);
        // println!("the dominators of {:?} are {:?}", root, x.dominators(conflict));
        match i_d_index {
            Some(node_index) => match self.graph.node_weight(node_index) {
                Some(node) => match node.item {
                    NodeItem::Falsum => panic!("Dominating falsum"),
                    NodeItem::Literal(l) => Some(l),
                },
                None => panic!("No dominator"),
            },
            None => None,
        }
    }

    pub fn immediate_dominators(
        &mut self,
        clause: &impl Clause,
        choice_literal: Literal,
    ) -> Option<Literal> {
        let falsum = self.add_temporary_falsum(clause);
        let root = self.get_literal(choice_literal);
        let dominators = simple_fast(&self.graph, root);
        let immediate_dominator = dominators.immediate_dominator(falsum);
        self.remove_node(falsum);
        match immediate_dominator {
            Some(i_d) => match self.graph.node_weight(i_d).unwrap().item {
                NodeItem::Literal(literal) => Some(literal),
                NodeItem::Falsum => None,
            },
            None => None,
        }
    }

    pub fn implying_clauses(&self, literal: Literal) -> impl Iterator<Item = ClauseId> + '_ {
        self.graph
            .edges_directed(self.get_literal(literal), petgraph::Direction::Incoming)
            .filter_map(|edge| match edge.weight().source {
                ImplicationSource::StoredClause(clause_id) => Some(clause_id),
                _ => None,
            })
    }

    pub fn literal_history(&self, literal: Literal) {
        let incoming_edges = self
            .graph
            .edges_directed(self.get_literal(literal), petgraph::Direction::Incoming);
        for edge in incoming_edges {
            println!("{:?}", edge);
        }
    }
}

impl<'borrow, 'graph> ImplicationGraph {
    pub fn remove_literals<I>(&'borrow mut self, literals: I)
    where
        I: Iterator<Item = Literal>,
    {
        literals.for_each(|literal| self.remove_literal(literal))
    }

    pub fn remove_level(&mut self, level: &Level) {
        self.remove_literals(level.literals());
        if self
            .graph
            .node_weights()
            .filter(|n| n.level == level.index())
            .count()
            != 0
        {
            panic!("Failed to remove all nodes at level {}", level.index());
        };
    }

    /*
    Given a clause and a level, the resolution candidates are those literals set at the level which conflict with some literal in the clause
     */
    pub fn naive_resolution_candidates<'clause>(
        &'borrow self,
        clause: &'borrow impl Clause,
        level: usize,
    ) -> impl Iterator<Item = (ClauseId, Literal)> + 'borrow {
        clause
            .literals()
            .filter_map(move |literal| {
                // get the node index of the literal
                let the_node_index = self.variable_indicies[literal.v_id].expect("Missing node");
                // get the literal as a node
                let the_literal_node = self
                    .graph
                    .node_weight(the_node_index)
                    .expect("Missing node");
                if the_literal_node.level == level {
                    match the_literal_node.item {
                        NodeItem::Falsum => panic!("hek"),
                        NodeItem::Literal(l) => {
                            // if there's some incoming edge, then there's going to be a possibility to resolve
                            if true {
                                //  || l.polarity != literal.polarity {
                                Some(
                                    self.graph
                                        .edges_directed(
                                            the_node_index,
                                            petgraph::Direction::Incoming,
                                        )
                                        .filter_map(move |edge| match edge.weight().source {
                                            ImplicationSource::StoredClause(clause_id) => {
                                                Some((clause_id, l))
                                            }
                                            _ => None,
                                        }),
                                )
                            } else {
                                None
                            }
                        }
                    }
                } else {
                    None
                }
            })
            .flatten()
    }
}
