use crate::structures::{Clause, ClauseId, Formula, Literal, Valuation, VariableId, Level};
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
    backward_edges: Vec<EdgeId>,
    forward_edges: Vec<EdgeId>,
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
    paths: Vec<Vec<usize>>,
    next_path_id: usize,
}

impl ImpGraphPaths {
    pub fn new_empty() -> Self {
        ImpGraphPaths {
            from: vec![],
            paths: vec![],
            next_path_id: 0,
        }
    }

    pub fn new_empty_of_size(size: usize, from: Vec<VariableId>) -> Self {
        ImpGraphPaths {
            from,
            paths: vec![vec![]; size],
            next_path_id: 0,
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

    pub fn extend(&mut self, from: BTreeSet<VariableId>, ante: &Clause, cseq: VariableId) {
        for literal in &ante.literals {
            if literal.v_id != cseq && from.contains(&literal.v_id) {
                self.edges.push(ImpGraphEdge {
                    from: literal.v_id,
                    to: cseq,
                    clause_id: ante.id,
                })
            }
        }
    }

    pub fn annotate_node_edges(&mut self, forwards: bool, backwards: bool) {
        for (edge_id, edge) in self.edges.iter().enumerate() {
            if forwards {
                if let Some(node) = self.nodes.get_mut(edge.from as usize) {
                    if !node.forward_edges.contains(&edge_id) {
                        node.forward_edges.push(edge_id);
                    }
                }
            }

            if backwards {
                if let Some(node) = self.nodes.get_mut(edge.to as usize) {
                    if !node.backward_edges.contains(&edge_id) {
                        node.backward_edges.push(edge_id);
                    }
                }
            }
        }
    }

    /* a dfs over the graph where each terminal node is given a unique number and any intermediate node has all the numbers of it's associated terminal nodes
    only terminal nodes are considered, as if non terminal then the literal was unit from the decision and so must be passed through anyway

    could be improved by ignoring edges which differ only by clause id
    */
    pub fn trace_implication_paths(&mut self, to: VariableId, from: Vec<VariableId>) {
        // work backwards from terminal nodes
        let mut path_obj = ImpGraphPaths::new_empty_of_size(
            self.formula.var_count(),
            from, // .iter()
                  // .filter(|&&v_id| self.nodes[v_id as usize].forward_edges.is_empty())
                  // .cloned()
                  // .collect::<Vec<_>>(),
        );

        fn helper(
            g: &ImpGraph,
            node_index: usize,
            p_obj: &mut ImpGraphPaths,
            from: VariableId,
        ) -> Vec<usize> {
            let the_node = &g.nodes[node_index];
            match &the_node.backward_edges.is_empty() {
                true => {
                    if the_node.v_id == from {
                        let path_id = p_obj.next_path_id;
                        p_obj.next_path_id += 1;
                        p_obj.paths[node_index].push(path_id);
                        vec![path_id]
                    } else {
                        vec![]
                    }
                }
                false => {
                    let mut sub_paths = vec![];
                    for edge in &the_node.backward_edges {
                        let child: usize = g.edges[*edge].from.try_into().unwrap();
                        let new_paths = helper(g, child, p_obj, from);
                        sub_paths.extend(new_paths);
                    }
                    for sub_path_id in &sub_paths {
                        p_obj.paths[node_index].push(*sub_path_id);
                    }
                    sub_paths
                }
            }
        }

        let variables = path_obj.from.iter().cloned().collect::<BTreeSet<_>>();

        for v in variables {
            helper(self, v as usize, &mut path_obj, to);
        }
        self.implication_paths = path_obj
    }

    pub fn unique_point<T: Valuation + std::fmt::Debug>(&self, clause: &Clause, val: T, level: &Level) {
        let possible_nodes = self.implication_paths.paths.iter().enumerate();
        for (id, p_info) in self.implication_paths.paths.iter().enumerate() {
            if !p_info.is_empty() && p_info.len() == self.implication_paths.next_path_id {
                // println!("{:?}", self.nodes[id]);
                // let v = self.nodes[id].v_id;
                // println!("{:?}", self.nodes[v as usize]);
            }
        }
        if self
            .implication_paths
            .paths
            .iter()
            .all(|b| b.is_empty())
        {
            println!("e so {}", level.choices.first().unwrap());
            println!("e so {}", clause);
            println!("e so {:?}", val);
            println!("e so {}", self);
            for (i, edge) in self.edges.iter().enumerate() {
                println!("{i} - {:?}", edge)
            }


            panic!();
        }

        // let possible_nodes2 = self
        //     .implication_paths
        //     .paths
        //     .iter()
        //     .filter(|x| !x.is_empty())
        //     .map(|y| y.clone().len())
        //     .collect::<Vec<_>>();
        // println!(
        //     "{} & possible nodes2: {:?}",
        //     self.implication_paths.next_path_id, possible_nodes2
        // );
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
            backward_edges: vec![],
            forward_edges: vec![],
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

impl std::fmt::Display for ImpGraph<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "#[{:?}] ", self.nodes)?;
        write!(f, "#[{:?}] ", self.edges)?;
        Ok(())
    }
}
