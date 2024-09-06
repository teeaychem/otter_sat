use crate::structures::{ClauseId, Formula, Literal, VariableId};
use std::collections::{BTreeSet, VecDeque};

// Implication graph

#[derive(Clone, Debug)]
pub struct ImpGraph<'formula> {
    formula: &'formula Formula,
    nodes: Vec<ImpGraphNode>,
    edges: Vec<ImpGraphEdge>,
    implication_paths: ImpGraphPaths,
}

#[derive(Clone, Debug)]
pub struct ImpGraphNode {
    v_id: VariableId,
    backward_edges: Option<Vec<EdgeId>>,
    forward_edges: Option<Vec<EdgeId>>,
}

pub type EdgeId = usize;

#[derive(Clone, Debug)]
pub struct ImpGraphEdge {
    from: VariableId,
    to: VariableId,
    clause_id: ClauseId,
}

#[derive(Clone, Debug)]
struct ImpGraphPaths {
    from: Vec<VariableId>,
    paths: Vec<Option<Vec<usize>>>,
    count: usize,
}

impl ImpGraphPaths {
    pub fn new_empty() -> Self {
        ImpGraphPaths {
            from: vec![],
            paths: vec![],
            count: 0,
        }
    }

    pub fn new_empty_of_size(size: usize, from: Vec<VariableId>) -> Self {
        ImpGraphPaths {
            from,
            paths: vec![None; size],
            count: 0,
        }
    }
}

impl ImpGraph<'_> {
    pub fn for_formula(formula: &Formula) -> ImpGraph {
        ImpGraph {
            formula,
            edges: vec![],
            implication_paths: ImpGraphPaths::new_empty(),
            nodes: (0..formula.var_count())
                .map(|i| ImpGraphNode::new(i as u32))
                .collect::<Vec<_>>(),
        }
    }

    pub fn add_edge(&mut self, edge: ImpGraphEdge) {
        self.edges.push(edge)
    }

    pub fn annotate_node_edges(&mut self, forwards: bool, backwards: bool) {
        for (edge_id, edge) in self.edges.iter().enumerate() {
            if forwards {
                if let Some(node) = self.nodes.get_mut(edge.from as usize) {
                    if let Some(ref mut vec) = node.forward_edges {
                        if !vec.contains(&edge_id) {
                            vec.push(edge_id);
                        }
                    } else {
                        self.nodes[edge.from as usize].forward_edges = Some(Vec::from([edge_id]));
                    }
                }
            }

            if backwards {
                if let Some(node) = self.nodes.get_mut(edge.to as usize) {
                    if let Some(ref mut vec) = node.backward_edges {
                        if !vec.contains(&edge_id) {
                            vec.push(edge_id);
                        }
                    } else {
                        self.nodes[edge.to as usize].backward_edges = Some(Vec::from([edge_id]));
                    }
                }
            }
        }
    }

    /* a dfs over the graph where each terminal node is given a unique number and any intermediate node has all the numbers of it's associated terminal nodes
    only terminal nodes are considered, as if non terminal then the literal was unit from the decision and so must be passed through anyway

    could be improved by ignoring edges which differ only by clause id
    */
    pub fn trace_implication_paths(&mut self, from: Vec<Literal>) {
        let mut path_obj = ImpGraphPaths::new_empty_of_size(
            self.formula.var_count(),
            from.iter()
                .map(|l| l.v_id)
                .filter(|&v_id| self.nodes[v_id as usize].forward_edges.is_none())
                .collect::<Vec<_>>(),
        );

        fn helper(g: &ImpGraph, n: usize, p: usize, p_obj: &mut ImpGraphPaths) -> Vec<usize> {
            let mut sub_paths = vec![p];
            let mut new_paths = vec![];
            if let Some(edges) = &g.nodes[n].backward_edges {
                for (index, edge) in edges.iter().enumerate() {
                    let child: usize = g.edges[*edge].from.try_into().unwrap();
                    if index > 0 { // only add to the count when branching
                        p_obj.count += 1;
                        new_paths.push(p_obj.count);
                    }
                    sub_paths.extend(helper(g, child, p_obj.count, p_obj));
                }
            }
            if p_obj.paths[n].is_some() {
                sub_paths
                    .iter()
                    .for_each(|p| p_obj.paths[n].as_mut().expect("hek").push(*p));
            } else {
                p_obj.paths[n] = Some(sub_paths);
            }

            new_paths
        }

        let variables = path_obj.from.iter().cloned().collect::<BTreeSet<_>>();

        for v in variables {
            path_obj.count += 1;
            helper(self, v as usize, path_obj.count, &mut path_obj);
        }
        self.implication_paths = path_obj
    }

    pub fn unique_point(&self) {

        let possible_nodes = self.implication_paths.paths.iter().filter(|p_info| p_info.as_ref().is_some_and(|x| x.len() == self.implication_paths.count)).cloned().collect::<Vec<_>>();
        println!("possible nodes: {:?}", possible_nodes);
        let mut found_points: Vec<ImpGraphNode> = vec![];
        let mut queue: VecDeque<&ImpGraphNode> = VecDeque::new();

//         // set up the queue
//         self.implication_paths.from.iter().for_each(|literal| {
//             if let Some(node) = self.nodes.iter().find(|n| n.v_id == literal.v_id) {
//                 queue.push_front(node);
//             }
//         });

//         loop {
//             if queue.is_empty() {
//                 break;
//             }
//             if let Some(node) = queue.pop_back() {
//                 if node.
// }




// }
    }

    pub fn generate_details(&mut self) {
        self.annotate_node_edges(true, true);
    }
}

impl ImpGraphNode {
    pub fn new(v_id: VariableId) -> Self {
        ImpGraphNode {
            v_id,
            backward_edges: None,
            forward_edges: None,
        }
    }
}

impl ImpGraphEdge {
    pub fn new(from: VariableId, clause_id: ClauseId, to: VariableId) -> Self {
        ImpGraphEdge {
            from,
            clause_id,
            to,
        }
    }
}
