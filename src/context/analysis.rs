use crate::{
    context::{
        config,
        resolution_buffer::{ResolutionBuffer, Status as BufferStatus},
        store::ClauseKey,
        Context, GraphLiteral, ImplicationGraphNode, Status as SolveStatus,
    },
    structures::{
        clause::{stored::Source as ClauseSource, Clause},
        literal::{Literal, Source as LiteralSource},
        variable::variable_store::VariableStore,
    },
};

use petgraph::{visit, Direction};
use std::collections::BTreeSet;
use std::ops::Deref;

impl Context {
    pub fn conflict_analysis(
        &mut self,
        clause_key: ClauseKey,
        vsids_variant: config::VSIDS,
        stopping_criteria: config::StoppingCriteria,
        activity: f32,
        subsumption: bool,
    ) -> SolveStatus {
        log::trace!("Fix @ {}", self.level().index());
        if self.level().index() == 0 {
            return SolveStatus::NoSolution;
        }
        let conflict_clause = self.stored_clauses.retreive(clause_key);
        let conflict_index = conflict_clause.get_node_index();
        log::trace!("Clause {conflict_clause}");

        // this could be made persistent, but tying it to the solve requires a cell and lots of unsafe
        let mut the_buffer = ResolutionBuffer::from_variable_store(&self.variables);

        the_buffer.reset_with(&self.variables);
        the_buffer.clear_literals(self.level().literals());
        the_buffer.set_inital_clause(&conflict_clause.deref(), clause_key);

        if let Some(asserted) = the_buffer.asserts() {
            // check to see if missed
            let missed_level = self.backjump_level(conflict_clause.literal_slice());
            self.backjump(missed_level);
            self.literal_update(asserted, LiteralSource::StoredClause(conflict_index));
            self.consequence_q.push_back(asserted);

            SolveStatus::MissedImplication
        } else {
            // resolve

            let ob_clone = self
                .level()
                .observations
                .iter()
                .rev()
                .cloned()
                .collect::<Vec<_>>();
            match the_buffer.resolve_with(
                ob_clone.iter(),
                &mut self.stored_clauses,
                &self.implication_graph,
                &self.variables,
                stopping_criteria,
                subsumption,
            ) {
                BufferStatus::FirstUIP | BufferStatus::Exhausted => {
                    the_buffer.strengthen_given(
                        self.levels[0]
                            .observations
                            .iter()
                            .map(|(_, literal)| *literal),
                    );

                    let (asserted_literal, mut resolved_clause) = the_buffer.to_assertion_clause();
                    if let Some(assertion) = asserted_literal {
                        resolved_clause.push(assertion);
                    }

                    self.apply_VSIDS(&resolved_clause, &the_buffer, vsids_variant, activity);

                    let (source, index) = match resolved_clause.len() {
                        1 => {
                            self.backjump(0);

                            let graph_literal = GraphLiteral {
                                literal: asserted_literal.expect("literal not there"),
                            };
                            let literal_index = self
                                .implication_graph
                                .add_node(ImplicationGraphNode::Literal(graph_literal));

                            (LiteralSource::Resolution(literal_index), literal_index)
                        }
                        _ => {
                            let backjump_level =
                                self.backjump_level(resolved_clause.literal_slice());
                            self.backjump(backjump_level);

                            let stored_clause =
                                self.store_clause(resolved_clause, ClauseSource::Resolution);
                            let stored_index = stored_clause.get_node_index();

                            (LiteralSource::StoredClause(stored_index), stored_index)
                        }
                    };

                    for key in the_buffer.trail() {
                        let trail_clause = self.stored_clauses.retreive(*key);
                        let trail_index = trail_clause.get_node_index();
                        self.implication_graph.add_edge(index, trail_index, ());
                    }

                    let assertion = asserted_literal.expect("wuh");
                    self.literal_update(assertion, source);
                    self.consequence_q.push_back(assertion);
                    SolveStatus::AssertingClause
                }
            }
            // see if resolution can be strengthened
        }
    }

    #[allow(non_snake_case)]
    fn apply_VSIDS(
        &self,
        clause: &impl Clause,
        buffer: &ResolutionBuffer,
        variant: config::VSIDS,
        activity: f32,
    ) {
        match variant {
            config::VSIDS::Chaff => {
                for literal in clause.literal_slice() {
                    self.variables.get_unsafe(literal.index()).add_activity(activity);
                }
            }
            config::VSIDS::MiniSAT => {
                for index in buffer.variables_used() {
                    self.variables.get_unsafe(index).add_activity(activity);
                }
            }
        }
    }

    #[allow(clippy::single_match)]
    pub fn display_core(&self, conflict_key: ClauseKey) {
        println!();
        println!(
            "c An unsatisfiable core of {}:\n",
            self.config.formula_file.display()
        );

        let conflict_clause = self.stored_clauses.retreive(conflict_key);
        let conflict_index = conflict_clause.get_node_index();

        let mut basic_clause_set = BTreeSet::new();
        basic_clause_set.insert(conflict_index);
        for (source, _) in &self.level().observations {
            match source {
                LiteralSource::StoredClause(node_index) | LiteralSource::Resolution(node_index) => {
                    basic_clause_set.insert(*node_index);
                }
                _ => {}
            }
        }

        let mut core_set = BTreeSet::new();
        for node_index in &basic_clause_set {
            visit::depth_first_search(&self.implication_graph, Some(*node_index), |event| {
                match event {
                    visit::DfsEvent::Discover(index, _) => {
                        let outgoing = self
                            .implication_graph
                            .edges_directed(index, Direction::Outgoing);
                        if outgoing.count() == 0 {
                            let graph_node = self
                                .implication_graph
                                .node_weight(index)
                                .expect("missing node");
                            match graph_node {
                                ImplicationGraphNode::Clause(clause_weight) => {
                                    core_set.insert(clause_weight.key);
                                }
                                ImplicationGraphNode::Literal(_) => {}
                            }
                        }
                    }
                    _ => {}
                }
            });
        }

        for source_key in &core_set {
            let source_clause = self.stored_clauses.retreive(*source_key);
            let full_clause = source_clause.original_clause();
            println!("{}", full_clause.as_dimacs(&self.variables));
        }
    }

    /// Either the most recent decision level in the resolution clause prior to the current level or 0.
    /*
    The implementation works through the clause, keeping an ordered record of the top two decision levels: (second_to_top, top)
    the top decision level will be for the literal to be asserted when clause is learnt
     */
    fn backjump_level(&self, literals: &[Literal]) -> usize {
        let mut top_two = (None, None);
        for lit in literals {
            if let Some(dl) = self.variables.get_unsafe(lit.index()).decision_level() {
                match top_two {
                    (_, None) => top_two.1 = Some(dl),
                    (_, Some(t1)) if dl > t1 => {
                        top_two.0 = top_two.1;
                        top_two.1 = Some(dl);
                    }
                    (None, _) => top_two.0 = Some(dl),
                    (Some(t2), _) if dl > t2 => top_two.0 = Some(dl),
                    _ => {}
                }
            }
        }

        match top_two {
            (None, _) => 0,
            (Some(second_to_top), _) => second_to_top,
        }
    }
}
