use crate::{
    config::{self, ActivityConflict, Config},
    context::{
        resolution_buffer::{ResolutionBuffer, Status as BufferStatus},
        store::ClauseKey,
        Context, GraphLiteral, ImplicationGraphNode, Status as SolveStatus,
    },
    structures::{
        clause::{stored::Source as ClauseSource, Clause},
        literal::{Literal, Source as LiteralSource},
        variable::list::VariableList,
    },
};

use petgraph::graph::NodeIndex;
use petgraph::{visit, Direction};
use std::collections::BTreeSet;
use std::ops::Deref;

impl Context {
    pub fn conflict_analysis(&mut self, clause_key: ClauseKey, config: &Config) -> SolveStatus {
        log::trace!("Fix @ {}", self.level().index());
        if self.level().index() == 0 {
            return SolveStatus::NoSolution(clause_key);
        }
        let conflict_clause = self.clause_store.retreive(clause_key);
        let conflict_index = conflict_clause.node_index();
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
            match self.variables.set_value(
                asserted,
                unsafe { self.levels.get_unchecked_mut(missed_level) },
                LiteralSource::Clause(conflict_index),
            ) {
                Ok(_) => {}
                Err(_) => return SolveStatus::NoSolution(clause_key),
            };
            self.variables.push_back_consequence(asserted);

            SolveStatus::MissedImplication(clause_key)
        } else {
            // resolve
            match the_buffer.resolve_with(
                unsafe {
                    // to avoid borrowing via the context, unsafe as the level index is either 0 or the last item of levels
                    self.levels
                        .get_unchecked(self.level().index())
                        .observations()
                },
                &mut self.clause_store,
                &self.implication_graph,
                &self.variables,
                config,
            ) {
                BufferStatus::FirstUIP | BufferStatus::Exhausted => {
                    the_buffer.strengthen_given(
                        self.levels[0]
                            .observations()
                            .iter()
                            .map(|(_, literal)| *literal),
                    );

                    let (asserted_literal, mut resolved_clause) = the_buffer.to_assertion_clause();
                    if let Some(assertion) = asserted_literal {
                        resolved_clause.push(assertion);
                    }

                    self.apply_VSIDS(&resolved_clause, &the_buffer, config);

                    let asserted_literal = asserted_literal.expect("literal not there");

                    let index = match resolved_clause.len() {
                        1 => {
                            self.backjump(0);

                            let graph_literal = GraphLiteral {
                                literal: asserted_literal,
                            };
                            let literal_index = self
                                .implication_graph
                                .add_node(ImplicationGraphNode::Literal(graph_literal));

                            match self.variables.set_value(
                                asserted_literal,
                                unsafe { self.levels.get_unchecked_mut(0) },
                                LiteralSource::Resolution(literal_index),
                            ) {
                                Ok(_) => {}
                                Err(_) => return SolveStatus::NoSolution(clause_key),
                            };

                            literal_index
                        }
                        _ => {
                            let backjump_level_index =
                                self.backjump_level(resolved_clause.literal_slice());
                            self.backjump(backjump_level_index);

                            let stored_clause =
                                self.store_clause(resolved_clause, ClauseSource::Resolution);
                            let stored_index = stored_clause.node_index();

                            match self.variables.set_value(
                                asserted_literal,
                                unsafe { self.levels.get_unchecked_mut(backjump_level_index) },
                                LiteralSource::Clause(stored_index),
                            ) {
                                Ok(_) => {}
                                Err(_) => return SolveStatus::NoSolution(clause_key),
                            };

                            stored_index
                        }
                    };

                    for key in the_buffer.trail() {
                        let trail_clause = self.clause_store.retreive(*key);
                        let trail_index = trail_clause.node_index();
                        self.implication_graph.add_edge(index, trail_index, ());
                    }

                    self.variables.push_back_consequence(asserted_literal);
                    SolveStatus::AssertingClause(clause_key)
                }
            }
        }
    }

    #[allow(non_snake_case)]
    fn apply_VSIDS(&mut self, clause: &impl Clause, buffer: &ResolutionBuffer, config: &Config) {
        let activity = config.activity_conflict;
        // let MAX_SCORE = 1e150;
        let MAX_SCORE = (2.0 as ActivityConflict).powi(512);

        let mut rescore = false;
        for literal in clause.literal_slice() {
            if self.variables.activity_of(literal.index()) + activity > MAX_SCORE {
                rescore = true;
                break;
            }
        }
        if rescore {
            self.variables.rescore_activity()
        }

        match config.vsids_variant {
            config::VSIDS::Chaff => {
                for literal in clause.literal_slice() {
                    let literal_index = literal.index();
                    self.variables.bump_activity(literal_index);
                }
            }
            config::VSIDS::MiniSAT => {
                for index in buffer.variables_used() {
                    self.variables.bump_activity(index);
                }
            }
        }

        self.variables.decay_activity(config);
    }

    #[allow(clippy::single_match)]
    pub fn display_core(&self, conflict_key: ClauseKey) {
        println!();
        println!("c An unsatisfiable core of the formula:\n",);

        let conflict_clause = self.clause_store.retreive(conflict_key);
        let conflict_index = conflict_clause.node_index();

        let mut basic_clause_set = BTreeSet::new();
        basic_clause_set.insert(conflict_index);
        for (source, _) in self.level().observations() {
            match source {
                LiteralSource::Clause(node_index) | LiteralSource::Resolution(node_index) => {
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

        for (_, literal) in self.level().observations() {
            match literal.polarity() {
                true => println!("{} 0", self.variables.external_name(literal.index())),
                false => println!("-{} 0", self.variables.external_name(literal.index())),
            };
        }

        for source_key in &core_set {
            let source_clause = self.clause_store.retreive(*source_key);
            let full_clause = source_clause.original_clause();
            println!("{}", full_clause.as_dimacs(&self.variables));
        }
    }

    pub fn literal_derivation(&self, index: NodeIndex) {
        let mut core_set = BTreeSet::new();

        visit::depth_first_search(&self.implication_graph, Some(index), |event| {
            if let visit::DfsEvent::Discover(index, _) = event {
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
        });

        for key in core_set {
            let clause = self.clause_store.retreive(key);
            println!("{}", clause.as_string());
        }
    }

    /// The backjump level for a slice of an asserting slice of literals/clause
    /// I.e. returns the second highest decision level from the given literals, or 0
    /*
    The implementation works through the clause, keeping an ordered record of the top two decision levels: (second_to_top, top)
    the top decision level will be for the literal to be asserted when clause is learnt
     */
    // TODO: could be duplicated/genralised as part of resolution, tho isn't very hot at the moment
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
