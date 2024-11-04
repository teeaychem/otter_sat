use crate::{
    config::{self, Config, StoppingCriteria},
    context::{
        resolution_buffer::{BufferStatus, ResolutionBuffer},
        store::ClauseKey,
        Context,
    },
    structures::{
        clause::{stored::ClauseSource, Clause},
        literal::Literal,
        variable::list::VariableList,
    },
};

use std::ops::Deref;

pub enum AnalysisResult {
    MissedImplication(ClauseKey, Literal),
    Proof(ClauseKey, Literal),
    FundamentalConflict(ClauseKey),
    QueueConflict(ClauseKey),
    AssertingClause(ClauseKey, Literal),
}

pub enum AnalysisIssue {
    ResolutionStore,
}

use crate::log::targets::ANALYSIS as LOG_ANALYSIS;
impl Context {
    pub fn conflict_analysis(
        &mut self,
        clause_key: ClauseKey,
        config: &Config,
    ) -> Result<AnalysisResult, AnalysisIssue> {
        log::trace!(target: LOG_ANALYSIS, "Analysis called on {clause_key} at level {}", self.levels.index());
        if self.levels.index() == 0 {
            return Ok(AnalysisResult::FundamentalConflict(clause_key));
        }
        let conflict_clause = self.clause_store.get(clause_key);
        // log::trace!(target: LOG_ANALYSIS, "Clause {conflict_clause}");

        if let config::VSIDS::Chaff = config.vsids_variant {
            self.variables.apply_VSIDS(
                conflict_clause
                    .literal_slice()
                    .iter()
                    .map(|literal| literal.index()),
                config,
            );
        }

        // this could be made persistent, but tying it to the solve requires a cell and lots of unsafe
        let mut the_buffer = ResolutionBuffer::from_variable_store(&self.variables);

        the_buffer.clear_literals(self.levels.top().literals());
        the_buffer.set_inital_clause(&conflict_clause.deref(), clause_key);

        if let Some(asserted_literal) = the_buffer.asserts() {
            Ok(AnalysisResult::MissedImplication(
                clause_key,
                asserted_literal,
            ))
        } else {
            let buffer_status = the_buffer.resolve_with(
                self.levels.top(),
                &mut self.clause_store,
                &mut self.variables,
                config,
            );
            match buffer_status {
                BufferStatus::FirstUIP => {}
                BufferStatus::Exhausted => {
                    assert_ne!(config.stopping_criteria, StoppingCriteria::FirstUIP)
                }
            }
            the_buffer.strengthen_given(self.proven_literals());

            let (asserted_literal, mut resolved_clause) = the_buffer.to_assertion_clause();
            if let Some(assertion) = asserted_literal {
                resolved_clause.push(assertion);
            }

            if let config::VSIDS::MiniSAT = config.vsids_variant {
                self.variables
                    .apply_VSIDS(the_buffer.variables_used(), config);
            }

            for key in the_buffer.trail() {
                self.clause_store.bump_activity(*key, config);
            }

            let the_literal = match asserted_literal {
                None => panic!("failed to resolve to an asserting clause"),
                Some(literal) => literal,
            };

            match resolved_clause.len() {
                0 => {
                    panic!("oh")
                }
                1 => {
                    self.proofs.push((the_literal, the_buffer.trail().to_vec()));
                    Ok(AnalysisResult::Proof(clause_key, the_literal))
                }
                _ => {
                    let Ok(clause) = self.store_clause(
                        resolved_clause,
                        Vec::default(),
                        ClauseSource::Resolution,
                        Some(the_buffer.trail().to_vec()),
                    ) else {
                        return Err(AnalysisIssue::ResolutionStore);
                    };
                    Ok(AnalysisResult::AssertingClause(clause.key(), the_literal))
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
    pub fn backjump_level(&self, literals: &[Literal]) -> usize {
        let mut top_two = (None, None);
        for lit in literals {
            let Some(dl) = self.variables.get_unsafe(lit.index()).decision_level() else {
                panic!("could not get decision level")
            };

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

        match top_two {
            (None, _) => 0,
            (Some(second_to_top), _) => second_to_top,
        }
    }
}
