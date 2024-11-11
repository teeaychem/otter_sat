use crate::{
    config::{self, Config, StoppingCriteria},
    context::{
        resolution_buffer::{BufOk, ResolutionBuffer},
        stores::ClauseKey,
        Context,
    },
    structures::{
        literal::{Literal, LiteralSource, LiteralTrait},
        variable::list::VariableList,
    },
    types::{clause::ClauseSource, errs::AnalysisError},
};

use std::ops::Deref;

pub enum AnalysisResult {
    MissedImplication(ClauseKey, Literal),
    Proof(ClauseKey, Literal),
    FundamentalConflict,
    QueueConflict,
    AssertingClause(ClauseKey, Literal),
}

#[allow(unused_imports)]
use crate::log::targets::ANALYSIS as LOG_ANALYSIS;

impl Context {
    pub fn conflict_analysis(
        &mut self,
        clause_key: ClauseKey,
        config: &Config,
    ) -> Result<AnalysisResult, AnalysisError> {
        log::trace!(target: LOG_ANALYSIS, "Analysis of {clause_key} at level {}", self.levels.decision_made());
        if !self.levels.decision_made() {
            return Ok(AnalysisResult::FundamentalConflict);
        }

        log::trace!(target: LOG_ANALYSIS, "Clause {clause_key}");

        if let config::VSIDS::Chaff = config.vsids_variant {
            self.variables.apply_VSIDS(
                self.clause_store
                    .get(clause_key)
                    .expect("missing clause")
                    .deref()
                    .iter()
                    .map(|literal| literal.index()),
                config,
            );
        }

        // this could be made persistent, but tying it to the solve may require a cell and lots of unsafe
        let mut the_buffer =
            ResolutionBuffer::from_variable_store(&self.variables, self.sender.clone());

        the_buffer.clear_literal(self.levels.current_choice());
        for (_, lit) in self.levels.current_consequences() {
            the_buffer.clear_literal(*lit);
        }

        match the_buffer.resolve_with(
            clause_key,
            &self.levels,
            &mut self.clause_store,
            &mut self.variables,
            config,
        ) {
            Ok(BufOk::Proof) => {}
            Ok(BufOk::FirstUIP) => {}
            Ok(BufOk::Exhausted) => {
                if config.stopping_criteria == StoppingCriteria::FirstUIP {
                    return Err(AnalysisError::FailedStoppingCriteria);
                }
            }
            Ok(BufOk::Missed(k, l)) => {
                return Ok(AnalysisResult::MissedImplication(k, l));
            }
            Err(_buffer_error) => {
                return Err(AnalysisError::Buffer);
            }
        }

        if let config::VSIDS::MiniSAT = config.vsids_variant {
            self.variables
                .apply_VSIDS(the_buffer.variables_used(), config);
        }

        for key in the_buffer.view_trail() {
            #[allow(clippy::single_match)]
            match key {
                ClauseKey::Learned(index, _) => self.clause_store.bump_activity(*index, config),
                _ => {}
            };
        }

        /*
        TODO: Alternative?
        Strengthening iterates through all the proven literals.
        This is skipped for a literal whose proof is to be noted
        This is also skipped for binary clauses, as if the other literal is proven the assertion will also be added as a proof, regardless
         */
        if the_buffer.clause_legnth() > 2 {
            the_buffer.strengthen_given(self.levels.proven_literals().iter());
        }

        let (asserted_literal, mut resolved_clause) = the_buffer.to_assertion_clause();
        // TODO: Revise this, maybe, as it means the watch is in the last place looked…
        if let Some(assertion) = asserted_literal {
            resolved_clause.push(assertion);
        }

        let the_literal = match asserted_literal {
            None => {
                log::error!(target: crate::log::targets::ANALYSIS, "Failed to resolve to an asserting clause");
                return Err(AnalysisError::NoAssertion);
            }
            Some(literal) => literal,
        };

        match resolved_clause.len() {
            0 => Err(AnalysisError::EmptyResolution),
            1 => {
                self.note_literal(the_literal, LiteralSource::Resolution(clause_key), unsafe {
                    the_buffer.take_trail()
                });

                Ok(AnalysisResult::Proof(clause_key, the_literal))
            }
            _ => {
                let Ok(clause_key) =
                    self.store_clause(resolved_clause, ClauseSource::Resolution, unsafe {
                        the_buffer.take_trail()
                    })
                else {
                    return Err(AnalysisError::ResolutionNotStored);
                };

                Ok(AnalysisResult::AssertingClause(clause_key, the_literal))
            }
        }
    }

    /// The second highest decision level from the given literals, or 0
    /// Aka. The backjump level for a slice of an asserting slice of literals/clause
    // Work through the clause, keeping an ordered record of the top two decision levels: (second_to_top, top)
    pub fn backjump_level(&self, literals: &[Literal]) -> Option<usize> {
        let mut top_two = (None, None);
        for literal in literals {
            let Some(dl) = self.variables.get_unsafe(literal.index()).decision_level() else {
                log::error!(target: crate::log::targets::BACKJUMP, "No decision level for {literal}");
                return None;
            };

            match top_two {
                (_, None) => top_two.1 = Some(dl),
                (_, Some(the_top)) if dl > the_top => {
                    top_two.0 = top_two.1;
                    top_two.1 = Some(dl);
                }
                (None, _) => top_two.0 = Some(dl),
                (Some(second_to_top), _) if dl > second_to_top => top_two.0 = Some(dl),
                _ => {}
            }
        }

        match top_two {
            (None, _) => Some(0),
            (Some(second_to_top), _) => Some(second_to_top),
        }
    }
}
