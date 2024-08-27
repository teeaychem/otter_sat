use crate::structures::{ClauseId, Literal, Solve, Valuation, VariableId};
use std::collections::BTreeSet;

// Implication graph

#[derive(Clone, Debug)]
pub struct ImpGraphNode {
    v_id: VariableId,
    backward_edges: Option<Vec<EdgeId>>,
    forward_edges: Option<Vec<EdgeId>>,
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

pub type EdgeId = usize;

#[derive(Clone, Debug)]
pub struct ImpGraphEdge {
    from: VariableId,
    to: VariableId,
    clause_id: ClauseId,
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

#[derive(Clone, Debug)]
pub struct ImpGraph {
    pub units: Vec<(ClauseId, Literal)>,
    edges: Vec<ImpGraphEdge>,
    valuation_size: usize,
    nodes: Vec<ImpGraphNode>,
    implication_paths: ImpGraphPaths,
}

impl ImpGraph {
    pub fn new(solve: &Solve) -> Self {
        ImpGraph {
            valuation_size: solve.valuation.len(),
            units: vec![],
            implication_paths: ImpGraphPaths::new_empty(),
            edges: vec![],
            nodes: (0..solve.valuation.len())
                .map(|i| ImpGraphNode::new(i as u32))
                .collect::<Vec<_>>(),
        }
    }

    pub fn for_level(solve: &Solve, level: usize) -> ImpGraph {
        let valuation = &solve.valuation_at_level(level);
        let the_units = solve.find_all_units_on(valuation, &mut BTreeSet::new());
        let units: Vec<Literal> = the_units
            .iter()
            .map(|(_clause, literal)| literal)
            .cloned()
            .collect();

        let relevant_ids = units
            .iter()
            .chain(solve.levels[level].literals().iter())
            .map(|l| l.v_id)
            .collect::<BTreeSet<_>>();

        let mut relevant_edges: Vec<ImpGraphEdge> = vec![];
        for (clause_id, to_literal) in &the_units {
            for from_literal in &solve.formula.borrow_clause_by_id(*clause_id).literals {
                if relevant_ids.contains(&from_literal.v_id) && from_literal != to_literal {
                    relevant_edges.push(ImpGraphEdge::new(
                        from_literal.v_id,
                        *clause_id,
                        to_literal.v_id,
                    ));
                }
            }
        }
        // let edges = the_units
        //     .iter()
        //     .flat_map(|(clause_id, to_literal)| {
        //         solve.clauses[*clause_id]
        //             .literals
        //             .iter()
        //             .filter(|&l| l != to_literal)
        //             .map(|from_literal| (from_literal.v_id, *clause_id, to_literal.v_id))
        //             .collect::<Vec<_>>()
        //     })
        //     .collect::<Vec<_>>();
        let mut the_graph = ImpGraph {
            valuation_size: valuation.size(),
            units: the_units,
            edges: relevant_edges,
            implication_paths: ImpGraphPaths::new_empty(),
            nodes: (0..valuation.size())
                .map(|i| ImpGraphNode::new(i as u32))
                .collect::<Vec<_>>(),
        };
        the_graph.annotate_node_edges(true, true);

        the_graph
    }

    fn annotate_node_edges(&mut self, forwards: bool, backwards: bool) {
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
            self.valuation_size,
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
                    if index > 0 {
                        // only add to the count when branching
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
}
