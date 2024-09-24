use petgraph::visit::{EdgeRef, NodeRef};
use petgraph::{
    graph::Graph,
    prelude::NodeIndex,
};

use crate::structures::ClauseId;
use std::rc::Rc;
use std::collections::VecDeque;

use super::StoredClause;

#[derive(Debug)]
enum Node {
    Clause(ClauseId),
    True,
}

#[derive(Debug)]
pub struct ResolutionGraph {
    graph: Graph<Node, usize>,
    the_true: NodeIndex,
    resolution_counter: usize,
}



impl ResolutionGraph {
    pub fn new() -> Self {
        let mut the_graph = Graph::<Node, usize>::new();
        let the_true = the_graph.add_node(Node::True);
        ResolutionGraph {
            graph: the_graph,
            the_true,
            resolution_counter: 0
        }
    }



    pub fn add_clause(&mut self, sc: Rc<StoredClause>) {
        let the_node = self.graph.add_node(Node::Clause(sc.id()));
        sc.set_nx(the_node);
        self.graph.add_edge(self.the_true, the_node, self.resolution_counter);
    }

    // pub fn add_resolution_by_ids(
    //     &mut self,
    //     from: impl Iterator<Item = ClauseId>,
    //     to: Rc<StoredClause>,
    // ) {
    //     let to_node = self.graph.add_node(Node::Clause(to.id()));
    //     to.set_nx(to_node);

    //     let mut bfs = Bfs::new(&self.graph, self.the_true);

    //     let mut the_ids = from.collect::<BTreeSet<_>>();
    //     let mut the_nodes = vec![];

    //     loop {
    //         if the_ids.is_empty() {
    //             break;
    //         }
    //         match bfs.next(&self.graph) {
    //             Some(nx) => {
    //                 if let Node::Clause(nc) = self.graph[nx] {
    //                     if the_ids.contains(&nc) {
    //                         the_nodes.push(nx);
    //                         the_ids.remove(&nc);
    //                     }
    //                 }
    //             }
    //             None => break,
    //         }
    //     }

    //     if !the_ids.is_empty() {
    //         panic!("Failed to find all antecedents of resolution");
    //     }

    //     for antecedent in the_nodes {
    //         self.graph.add_edge(antecedent, to_node, ());
    //     }
    // }

    // In particular, a clause id *could* be known in advance it's possible to add edges during resolution to save visiting a clause multiple times
    pub fn add_resolution(
        &mut self,
        from: impl Iterator<Item = Rc<StoredClause>>,
        to: Rc<StoredClause>,
    ) {
        let to_node = self.graph.add_node(Node::Clause(to.id()));
        to.set_nx(to_node);

        self.resolution_counter += 1;
        for antecedent in from {
            self.graph.add_edge(antecedent.nx(), to_node, self.resolution_counter);
        }
    }

    pub fn origins(&self, clause: NodeIndex) {
        let mut origins =  vec![];
        let mut q: VecDeque<NodeIndex> = VecDeque::new();
        q.push_back(clause);
        loop {
            if q.is_empty() {
                break;
            }

            let node = q.pop_front().expect("Ah, the queue was empty…");
            let incoming = self.graph.edges_directed(node, petgraph::Direction::Incoming);
            for edge in incoming {
                let from = self.graph.node_weight(edge.source()).expect("No incoming node");
                match from {
                    Node::True => origins.push(edge.target()),
                    Node::Clause(_) => q.push_back(edge.source())
                }
            }
        }
        println!("{:?}", origins)


    }
}
