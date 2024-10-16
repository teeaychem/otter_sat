use crate::{
    context::{
        config,
        resolution_buffer::{ResolutionBuffer, Status as BufferStatus},
        store::ClauseKey,
        Context, Status as SolveStatus,
    },
    structures::{
        clause::{
            stored::{Source as ClauseSource, StoredClause},
            Clause,
        },
        literal::{Literal, Source as LiteralSource},
    },
};

use std::{collections::VecDeque, ops::Deref};

impl Context {
    pub fn conflict_analysis(
        &mut self,
        clause_key: ClauseKey,
        vsids_variant: config::VSIDS,
        stopping_critera: config::StoppingCriteria,
        activity: f32,
    ) -> SolveStatus {
        log::trace!("Fix @ {}", self.level().index());
        if self.level().index() == 0 {
            return SolveStatus::NoSolution;
        }
        let conflict_clause = self.stored_clauses.retreive_unchecked(clause_key);
        log::trace!("Clause {conflict_clause}");

        // this could be made persistent, but tying it to the solve requires a cell and lots of unsafe
        let mut the_buffer = ResolutionBuffer::from_valuation(&self.valuation);

        the_buffer.reset_with(&self.valuation);
        the_buffer.clear_literals(self.level().literals());
        the_buffer.merge_clause(&conflict_clause.deref());

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
                            let clause_key = self.store_clause(
                                resolved_clause,
                                ClauseSource::Resolution(the_buffer.trail().to_vec()),
                            );

                            LiteralSource::StoredClause(clause_key)
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

    pub fn display_core(&self) {
        println!();
        println!("c An unsatisfiable core of the original formula:\n");
        let mut node_indicies = vec![];
        for (source, _) in &self.levels[0].observations {
            match source {
                LiteralSource::StoredClause(key) => node_indicies.push(*key),
                LiteralSource::Resolution(keys) => node_indicies.extend(keys),
                _ => {}
            }
        }
        let mut origins = self.extant_origins(node_indicies.iter().copied());
        origins.sort_unstable_by_key(|s| s.key());
        origins.dedup_by_key(|s| s.key());

        for clause in origins {
            println!("{}", clause.as_dimacs(&self.variables));
        }
        println!();
    }

    pub fn extant_origins(&self, clauses: impl Iterator<Item = ClauseKey>) -> Vec<&StoredClause> {
        let mut origin_nodes = vec![];
        let mut q = clauses.collect::<VecDeque<_>>();

        while !q.is_empty() {
            let clause_key = q.pop_front().expect("Ah, the queue was empty…");

            let stored_clause = self.stored_clauses.retreive_unchecked(clause_key);
            match stored_clause.source() {
                ClauseSource::Resolution(origins) => {
                    for antecedent in origins {
                        q.push_back(*antecedent);
                    }
                }
                ClauseSource::Formula => origin_nodes.push(stored_clause),
            }
        }
        origin_nodes
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
