use crate::structures::{Assignment, ClauseId, Literal, Solve, Valuation, VariableId};
use std::collections::BTreeSet;

// Implication graph

pub type EdgeId = usize;
pub type ImplicationGraphEdge = (VariableId, ClauseId, VariableId);

#[derive(Clone, Debug)]
pub struct ImplicationGraph {
    pub units: Vec<Literal>,
    backwards: Vec<Option<BTreeSet<EdgeId>>>, // indicies correspond to variables, indexed vec is for edges
    edges: Vec<ImplicationGraphEdge>,
}

impl ImplicationGraph {
    pub fn new(assignment: &Assignment) -> Self {
        ImplicationGraph {
            units: vec![],
            backwards: vec![None; assignment.valuation.len()],
            edges: vec![],
        }
    }

    pub fn for_level(assignment: &Assignment, level: usize, solve: &Solve) -> ImplicationGraph {
        let valuation = &assignment.valuation_at_level(level);
        let the_units = solve.find_all_units_on(valuation, &mut BTreeSet::new());
        let units: Vec<Literal> = the_units
            .iter()
            .map(|(_clause, literal)| literal)
            .cloned()
            .collect();

        let relevant_ids = units
            .iter()
            .chain(assignment.levels[level].literals().iter().cloned())
            .map(|l| l.v_id)
            .collect::<BTreeSet<_>>();

        let mut relevant_edges: Vec<ImplicationGraphEdge> = vec![];
        for (clause_id, to_literal) in the_units {
            for from_literal in &solve.clauses[clause_id].literals {
                if relevant_ids.contains(&from_literal.v_id) && *from_literal != to_literal {
                    relevant_edges.push((from_literal.v_id, clause_id, to_literal.v_id));
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
        let mut the_graph = ImplicationGraph {
            units,
            backwards: vec![None; valuation.size()],
            edges: relevant_edges,
        };
        the_graph.add_backwards();

        the_graph
    }

    pub fn add_backwards(&mut self) {
        for (edge_id, (_from_node, _clause_id, to_node)) in self.edges.iter().enumerate() {
            if let Some(Some(set)) = self.backwards.get_mut(*to_node as usize) {
                set.insert(edge_id);
            } else {
                self.backwards[*to_node as usize] = Some(BTreeSet::from([edge_id]));
            };
        }
    }
}
