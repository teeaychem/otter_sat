use petgraph::visit::{EdgeRef, NodeRef};
use petgraph::{graph::Graph, prelude::NodeIndex};

use crate::structures::{Clause, ClauseId, StoredClause};
use std::collections::{BTreeSet, VecDeque};
use std::rc::{Rc, Weak};

#[derive(Debug)]
enum Node {
    Clause(Weak<StoredClause>),
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
            resolution_counter: 0,
        }
    }

    pub fn add_clause(&mut self, sc: Rc<StoredClause>) {
        let the_node = self.graph.add_node(Node::Clause(Rc::downgrade(&sc)));
        sc.set_nx(the_node);
        self.graph
            .add_edge(self.the_true, the_node, self.resolution_counter);
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
        let to_node = self.graph.add_node(Node::Clause(Rc::downgrade(&to)));
        to.set_nx(to_node);

        self.resolution_counter += 1;
        for antecedent in from {
            self.graph
                .add_edge(antecedent.nx(), to_node, self.resolution_counter);
        }
    }

    pub fn extant_origins(&self, clauses: Vec<NodeIndex>) -> Vec<Rc<StoredClause>> {
        let mut origin_nodes = BTreeSet::new();

        let mut q: VecDeque<NodeIndex> = VecDeque::new();
        for clause in clauses {
            q.push_back(clause);
        }
        loop {
            if q.is_empty() {
                break;
            }

            let node = q.pop_front().expect("Ah, the queue was empty…");
            let incoming = self
                .graph
                .edges_directed(node, petgraph::Direction::Incoming);
            let mut deduction_id = None;
            for edge in incoming {
                let from = self
                    .graph
                    .node_weight(edge.source())
                    .expect("No incoming node");
                match from {
                    Node::True => {
                        origin_nodes.insert(node);
                    }

                    Node::Clause(_) => match deduction_id {
                        Some(d_id) if edge.weight().id() == d_id => q.push_back(edge.source()),
                        None => {
                            deduction_id = Some(edge.weight().id());
                            q.push_back(edge.source())
                        }
                        _ => {}
                    },
                }
            }
        }

        // for node in origin_nodes {
        //     match self.graph.node_weight(node) {
        //         Some(Node::Clause(weak_reference)) => {
        //             if let Some(stored_clause) = weak_reference.upgrade() {
        //                 origins.push(stored_clause);
        //             }
        //         }
        //         Some(Node::True) => panic!("the true has an incoming node…"),
        //         None => panic!("Node has disappeared"),
        //     }
        // }
        // for stored_clause in origins {
        //     println!("{}", stored_clause.clause().as_string())
        // }
        origin_nodes
            .iter()
            .flat_map(|nx| match self.graph.node_weight(*nx) {
                Some(Node::Clause(weak_reference)) => weak_reference.upgrade(),
                Some(Node::True) => panic!("the true has an incoming node…"),
                None => panic!("Node has disappeared"),
            }).collect()
    }
}
