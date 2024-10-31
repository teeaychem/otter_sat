use crate::{
    config::{self, Config},
    context::{
        resolution_buffer::{ResolutionBuffer, Status as BufferStatus},
        store::ClauseKey,
        Context, Status as SolveStatus,
    },
    structures::{
        clause::{stored::ClauseSource, Clause},
        literal::{Literal, LiteralSource},
        variable::{delegate::push_back_consequence, list::VariableList},
    },
};

use std::ops::Deref;

use super::core::ContextIssue;

impl Context {
    pub fn conflict_analysis(
        &mut self,
        clause_key: ClauseKey,
        config: &Config,
    ) -> Result<SolveStatus, ContextIssue> {
        log::trace!("Fix @ {}", self.levels.index());
        if self.levels.index() == 0 {
            return Ok(SolveStatus::NoSolution(clause_key));
        }
        let conflict_clause = self.clause_store.get(clause_key);
        let conflict_index = conflict_clause.key();
        log::trace!("Clause {conflict_clause}");

        if let config::VSIDS::Chaff = config.vsids_variant {
            self.variables.apply_VSIDS(
                conflict_clause
                    .literal_slice()
                    .iter()
                    .map(|literal| literal.index()),
                None,
                config,
            );
        }

        // this could be made persistent, but tying it to the solve requires a cell and lots of unsafe
        let mut the_buffer = ResolutionBuffer::from_variable_store(&self.variables);

        // the_buffer.reset_with(&self.variables);
        the_buffer.clear_literals(self.levels.top().literals());
        the_buffer.set_inital_clause(&conflict_clause.deref(), clause_key);

        if let Some(asserted) = the_buffer.asserts() {
            // check to see if missed
            let missed_level = self.backjump_level(conflict_clause.literal_slice());
            self.backjump(missed_level);
            push_back_consequence(
                &mut self.variables.consequence_q,
                asserted,
                LiteralSource::Missed(conflict_index, missed_level),
                self.levels.index(),
            );

            Ok(SolveStatus::MissedImplication(clause_key))
        } else {
            // resolve
            let observations = self.levels.top().observations();
            let buffer_status = the_buffer.resolve_with(
                observations,
                &mut self.clause_store,
                &self.variables,
                config,
            );
            match buffer_status {
                BufferStatus::FirstUIP | BufferStatus::Exhausted => {
                    // the_buffer.strengthen_given(self.proven_literals());

                    let (asserted_literal, mut resolved_clause) = the_buffer.to_assertion_clause();
                    if let Some(assertion) = asserted_literal {
                        resolved_clause.push(assertion);
                    }

                    if let config::VSIDS::MiniSAT = config.vsids_variant {
                        self.variables
                            .apply_VSIDS(the_buffer.variables_used(), None, config);
                        // alt hint Some(the_buffer.max_activity(&self.variables)),
                    }

                    let asserted_literal = asserted_literal.expect("literal not there");

                    match resolved_clause.len() {
                        1 => {
                            self.backjump(0);

                            self.proofs
                                .push((asserted_literal, the_buffer.trail().to_vec()));

                            push_back_consequence(
                                &mut self.variables.consequence_q,
                                asserted_literal,
                                LiteralSource::Resolution(clause_key),
                                self.levels.index(),
                            );
                        }
                        _ => {
                            let backjump_level_index =
                                self.backjump_level(resolved_clause.literal_slice());
                            self.backjump(backjump_level_index);

                            let stored_clause = self.store_clause(
                                resolved_clause,
                                ClauseSource::Resolution,
                                Some(the_buffer.trail().to_vec()),
                            )?;
                            let stored_index = stored_clause.key();
                            push_back_consequence(
                                &mut self.variables.consequence_q,
                                asserted_literal,
                                LiteralSource::Clause(stored_index),
                                self.levels.index(),
                            );
                        }
                    };
                    Ok(SolveStatus::AssertingClause(clause_key))
                }
            }
        }
    }

    /// The backjump level for a slice of an asserting slice of literals/clause
    /// I.e. returns the second highest decision level from the given literals, or 0
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
