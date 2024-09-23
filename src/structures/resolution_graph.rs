use petgraph::{
    graph::Graph,
    prelude::{Bfs, NodeIndex},
};

use crate::structures::ClauseId;
use std::collections::BTreeSet;

#[derive(Debug)]
enum Node {
    Clause(ClauseId),
    True,
}

#[derive(Debug)]
pub struct ResolutionGraph {
    graph: Graph<Node, ()>,
    the_true: NodeIndex,
}

impl ResolutionGraph {
    pub fn new() -> Self {
        let mut the_graph = Graph::<Node, ()>::new();
        let the_true = the_graph.add_node(Node::True);
        ResolutionGraph {
            graph: the_graph,
            the_true,
        }
    }

    pub fn add_clause(&mut self, id: ClauseId) {
        let the_node = self.graph.add_node(Node::Clause(id));
        self.graph.add_edge(self.the_true, the_node, ());
    }

    pub fn add_resolution_by_ids(&mut self, from: impl Iterator<Item = ClauseId>, to: ClauseId) {
        let to_node = self.graph.add_node(Node::Clause(to));

        let mut bfs = Bfs::new(&self.graph, self.the_true);

        let mut the_ids = from.collect::<BTreeSet<_>>();
        let mut the_nodes = vec![];

        loop {
            if the_ids.is_empty() {
                break;
            }
            match bfs.next(&self.graph) {
                Some(nx) => {
                    if let Node::Clause(nc) = self.graph[nx] {
                        if the_ids.contains(&nc) {
                            the_nodes.push(nx);
                            the_ids.remove(&nc);
                        }
                    }
                }
                None => break,
            }
        }

        if !the_ids.is_empty() {
            panic!("Failed to find all antecedents of resolution");
        }

        for antecedent in the_nodes {
            self.graph.add_edge(antecedent, to_node, ());
        }
    }

    // TODO: store node ids with clauses and refactor for efficient creation.
    // In particular, as a clause id *could* be known in advance it's possible to add edges during resolution to save visiting a clause multiple times
}
