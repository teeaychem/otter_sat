use crate::structures::{
    binary_resolution, solve::Solve, Clause, ClauseId, ClauseVec, Formula, Level, LevelIndex,
    Literal, StoredClause, Valuation, VariableId,
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
    level: LevelIndex,
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

    pub fn add_literal(&mut self, lit: Literal, lvl_idx: LevelIndex, conflict: bool) -> NodeIndex {
        log::trace!(target: target_graph!(), "?+ Literal @{lvl_idx}: {lit}");
        let index = self.graph.add_node(ImplicationNode {
            item: NodeItem::Literal(lit),
            level: lvl_idx,
        });
        if !conflict {
            self.variable_indicies[lit.v_id] = Some(index);
        }
        log::trace!(target: target_graph!(), "+Literal @{lvl_idx}: {lit}");
        index
    }

    pub fn get_or_make_literal(&mut self, lit: Literal, lvl_idx: LevelIndex) -> NodeIndex {
        let literal_index = match self.variable_indicies[lit.v_id] {
            Some(index) => {
                let the_node = self.graph.node_weight(index).unwrap();
                if the_node.level != lvl_idx {
                    panic!("Levels do not match!");
                }
                match the_node.item {
                    NodeItem::Falsum => panic!("get falsum"),
                    NodeItem::Literal(node_literal) => {
                        if node_literal.polarity != lit.polarity {
                            panic!("Graph polarity does not match for v_id {}.\nHave: {}\nRequested: {}", lit.v_id, node_literal.polarity, lit.polarity);
                        }
                        if node_literal.v_id != lit.v_id {
                            panic!("Variables do not match!");
                        }
                        index
                    }
                }
            }
            None => self.add_literal(lit, lvl_idx, false),
        };
        literal_index
    }

    pub fn get_literal(&self, lit: Literal) -> NodeIndex {
        let literal_index = match self.variable_indicies[lit.v_id] {
            Some(index) => {
                let the_node = self.graph.node_weight(index).unwrap();
                match the_node.item {
                    NodeItem::Falsum => panic!("Get falsum"),
                    NodeItem::Literal(node_literal) => {
                        if node_literal.polarity != lit.polarity {
                            panic!("Graph polarity does not match for v_id {}.\nHave: {}\nRequested: {}", lit.v_id, node_literal.polarity, lit.polarity);
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
                panic!("Unable to get {}", lit)
            }
        };
        literal_index
    }

    pub fn add_implication(
        &mut self,
        clause: impl Iterator<Item = Literal>,
        to: Literal,
        lvl_idx: LevelIndex,
        source: ImplicationSource,
    ) -> NodeIndex {
        let clause_vec = clause.collect::<Vec<_>>();

        log::warn!(target: target_graph!(), "+Implication {:?} -> {to}", clause_vec.as_string());
        let (consequent_index, description) =
            (self.get_or_make_literal(to, lvl_idx), "Implication");
        for antecedent in clause_vec.iter().filter(|a| a.v_id != to.v_id) {
            let edge_index = self.graph.add_edge(
                self.get_literal(*antecedent),
                consequent_index,
                ImplicationEdge { source },
            );
            let the_edge = self.graph.edge_weight(edge_index).unwrap();
            match the_edge.source {
                ImplicationSource::StoredClause(c) => {
                    log::debug!(target: target_graph!(), "+{description} @{lvl_idx}: {antecedent} --[{c}]-> {to}")
                }
                _ => {
                    log::debug!(target: target_graph!(), "+{description} @{lvl_idx}: {antecedent} --[{the_edge:?}]-> {to}")
                }
            }
        }
        // log::info!(target: target_graph!(), "+{description} @{level}: {} -> {to}", clause.as_string());
        consequent_index
    }

    fn add_temporary_falsum(&mut self, lits: impl Iterator<Item = Literal>) -> NodeIndex {
        log::warn!("Temporary falsum");
        let falsum = self.graph.add_node(ImplicationNode {
            level: 0, // as the falsum is temporary and the level is unimportant, it's fixed to 0
            item: NodeItem::Falsum,
        });
        for antecedent in lits {
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

    pub fn add_contradiction(&mut self, from: Literal, to: Literal, lvl_idx: LevelIndex) {
        let choice_index = self.get_literal(from);
        let contradiction_index = self.get_or_make_literal(to, lvl_idx);

        let edge_index = self.graph.add_edge(
            choice_index,
            contradiction_index,
            ImplicationEdge {
                source: ImplicationSource::Contradiction,
            },
        );
        log::debug!(target: target_graph!(), "+Contradiction @{lvl_idx} {from} --[{:?}]-> {to}", self.graph.edge_weight(edge_index));
    }

    pub fn remove_literal(&mut self, lit: Literal) {
        if let Some(index) = self.variable_indicies[lit.v_id] {
            if let Some(node) = self.graph.remove_node(index) {
                log::debug!(target: target_graph!(), "-{node:?}");
                self.variable_indicies[lit.v_id] = None;
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

    fn immediate_dominator(&self, root: NodeIndex, conflict: NodeIndex) -> Option<Literal> {
        let dominators = simple_fast(&self.graph, root);

        let i_d_index = dominators.immediate_dominator(conflict);
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
        lits: impl Iterator<Item = Literal>,
        choice_lit: Literal,
    ) -> Option<Literal> {
        log::warn!("Finding immediate dominator");
        let falsum = self.add_temporary_falsum(lits);
        let root = self.get_literal(choice_lit);
        let dominators = simple_fast(&self.graph, root);
        let immediate_dominator = dominators.immediate_dominator(falsum);
        self.remove_node(falsum);
        match immediate_dominator {
            Some(i_d) => {
                let candidate = &self.graph.node_weight(i_d).unwrap().item;
                match candidate {
                    NodeItem::Literal(literal) => Some(*literal),
                    NodeItem::Falsum => None,
                }
            }
            None => None,
        }
    }

    pub fn implying_clauses(&self, lit: Literal) -> impl Iterator<Item = ClauseId> + '_ {
        self.graph
            .edges_directed(self.get_literal(lit), petgraph::Direction::Incoming)
            .filter_map(|edge| match edge.weight().source {
                ImplicationSource::StoredClause(clause_id) => Some(clause_id),
                _ => None,
            })
    }

    pub fn literal_history(&self, lit: Literal) {
        let incoming_edges = self
            .graph
            .edges_directed(self.get_literal(lit), petgraph::Direction::Incoming);
        for edge in incoming_edges {
            println!("{:?}", edge);
        }
    }
}

impl<'borrow, 'graph> ImplicationGraph {
    fn get_literal_node(&self, lit: Literal) -> &ImplicationNode {
        let the_node_index = self.variable_indicies[lit.v_id].expect("Missing node");
        self.graph
            .node_weight(the_node_index)
            .expect("Missing node")
    }

    pub fn remove_literals(&'borrow mut self, lits: impl Iterator<Item = Literal>) {
        lits.for_each(|literal| self.remove_literal(literal))
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
    pub fn resolution_candidates_at_level<'clause>(
        &'borrow self,
        clause: &'borrow impl Clause,
        lvl_idx: LevelIndex,
    ) -> impl Iterator<Item = (ClauseId, Literal)> + 'borrow {
        clause
            .literals()
            .filter_map(move |literal| {
                let the_node_index = self.variable_indicies[literal.v_id].expect("Missing node");
                let the_literal_node = self
                    .graph
                    .node_weight(the_node_index)
                    .expect("Missing node");
                if the_literal_node.level == lvl_idx {
                    match the_literal_node.item {
                        NodeItem::Falsum => panic!("Literal from falsum"),
                        NodeItem::Literal(l) => Some(
                            self.graph
                                .edges_directed(the_node_index, petgraph::Direction::Incoming)
                                .filter_map(move |edge| match edge.weight().source {
                                    ImplicationSource::StoredClause(clause_id) => {
                                        Some((clause_id, l))
                                    }
                                    _ => None,
                                }),
                        ),
                    }
                } else {
                    None
                }
            })
            .flatten()
    }

    pub fn resolution_candidates_all<'clause>(
        &'borrow self,
        clause: &'borrow impl Clause,
    ) -> impl Iterator<Item = (ClauseId, Literal)> + 'borrow {
        clause
            .literals()
            .filter_map(move |literal| {
                let the_node_index = self.variable_indicies[literal.v_id].expect("Missing node");
                let the_literal_node = self
                    .graph
                    .node_weight(the_node_index)
                    .expect("Missing node");

                match the_literal_node.item {
                    NodeItem::Falsum => panic!("Literal from falsum"),
                    NodeItem::Literal(l) => Some(
                        self.graph
                            .edges_directed(the_node_index, petgraph::Direction::Incoming)
                            .filter_map(move |edge| match edge.weight().source {
                                ImplicationSource::StoredClause(clause_id) => Some((clause_id, l)),
                                _ => None,
                            }),
                    ),
                }
            })
            .flatten()
    }

    pub fn paths_between(
        &self,
        from: Literal,
        to: Literal,
    ) -> impl Iterator<Item = Vec<NodeIndex>> + '_ {
        let from_node = self.get_literal(from).id();
        let to_node = self.get_literal(to).id();
        petgraph::algo::all_simple_paths::<Vec<_>, _>(&self.graph, from_node, to_node, 0, None)
    }

    /// A path
    pub fn connecting_clauses(
        &self,
        mut path: impl Iterator<Item = &'borrow NodeIndex>,
    ) -> Vec<ClauseId> {
        let mut clause_ids = vec![];
        if let Some(mut from_node) = path.next() {
            let mut to_node;
            for node in path {
                to_node = node;
                let mut connecting_edges = self.graph.edges_connecting(*from_node, *to_node);
                let connecting_source = connecting_edges
                    .next()
                    .expect("No connecting edge")
                    .weight()
                    .source;
                match connecting_source {
                    ImplicationSource::StoredClause(clause_id) => {
                        clause_ids.push(clause_id);
                    }
                    _ => panic!("Edge without clause"),
                }
                from_node = to_node;
            }
        }
        clause_ids
    }
}
