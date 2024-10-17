use crate::{
    context::{
        config,
        resolution_buffer::{ResolutionBuffer, Status as BufferStatus},
        store::ClauseKey,
        Context, Status as SolveStatus,
    },
    structures::{
        clause::{stored::Source as ClauseSource, Clause},
        literal::{Literal, Source as LiteralSource},
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
        stopping_critera: config::StoppingCriteria,
        activity: f32,
        show_core: bool,
    ) -> SolveStatus {
        log::trace!("Fix @ {}", self.level().index());
        if self.level().index() == 0 {
            return SolveStatus::NoSolution;
        }
        let conflict_clause = self.stored_clauses.retreive(clause_key);
        log::trace!("Clause {conflict_clause}");

        // this could be made persistent, but tying it to the solve requires a cell and lots of unsafe
        let mut the_buffer = ResolutionBuffer::from_valuation(&self.valuation);

        the_buffer.reset_with(&self.valuation);
        the_buffer.clear_literals(self.level().literals());
        the_buffer.set_inital_clause(&conflict_clause.deref(), clause_key);

        if let Some(asserted) = the_buffer.asserts() {
            // check to see if missed
            let missed_level = self.backjump_level(conflict_clause.literal_slice());
            self.backjump(missed_level);
            self.literal_update(asserted, &LiteralSource::StoredClause(clause_key));
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
                &self.valuation,
                &self.variables,
                stopping_critera,
                show_core,
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

                    let source = match resolved_clause.len() {
                        1 => {
                            self.backjump(0);
                            LiteralSource::Resolution(the_buffer.trail().to_vec())
                        }
                        _ => {
                            let backjump_level =
                                self.backjump_level(resolved_clause.literal_slice());
                            self.backjump(backjump_level);
                            let resolved_key =
                                self.store_clause(resolved_clause, ClauseSource::Resolution);
                            let the_clause = self.stored_clauses.retreive(resolved_key);
                            let node_index = the_clause.get_node_index();

                            for key in the_buffer.trail() {
                                let trail_clause = self.stored_clauses.retreive(*key);
                                let trail_index = trail_clause.get_node_index();
                                self.implication_graph.add_edge(node_index, trail_index, ());
                            }

                            LiteralSource::StoredClause(resolved_key)
                        }
                    };
                    let assertion = asserted_literal.expect("wuh");
                    self.literal_update(assertion, &source);
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
                    self.get_variable(literal.index()).add_activity(activity);
                }
            }
            config::VSIDS::MiniSAT => {
                for index in buffer.variables_used() {
                    self.get_variable(index).add_activity(activity);
                }
            }
        }
    }

    #[allow(clippy::single_match)]
    pub fn display_core(&self, conflict_key: ClauseKey) {
        println!();
        println!("c An unsatisfiable core of {}:\n", self.config.formula_file.display());

        let mut basic_clause_set = BTreeSet::new();
        basic_clause_set.insert(conflict_key);
        for (source, _) in &self.level().observations {
            match source {
                LiteralSource::StoredClause(key) => {
                    basic_clause_set.insert(*key);
                }
                LiteralSource::Resolution(keys) => {
                    basic_clause_set.extend(keys);
                }
                _ => {}
            }
        }

        let mut core_set = BTreeSet::new();
        for key in &basic_clause_set {
            let clause = self.stored_clauses.retreive(*key);
            let node_index = clause.get_node_index();

            visit::depth_first_search(
                &self.implication_graph,
                Some(node_index),
                |event| match event {
                    visit::DfsEvent::Discover(index, _) => {
                        let outgoing = self
                            .implication_graph
                            .edges_directed(index, Direction::Outgoing);
                        if outgoing.count() == 0 {
                            let root_key = self
                                .implication_graph
                                .node_weight(index)
                                .expect("missing node")
                                .key;
                            core_set.insert(root_key);
                        }
                    }
                    _ => {}
                },
            );
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
            if let Some(dl) = self.get_variable(lit.index()).decision_level() {
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
