use crate::{
    config::{self, Config, StoppingCriteria},
    context::{
        resolution_buffer::{BufOk, ResolutionBuffer},
        stores::ClauseKey,
        unique_id::UniqueId,
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
    FundamentalConflict(ClauseKey),
    QueueConflict(ClauseKey),
    AssertingClause(ClauseKey, Literal),
}

use crate::log::targets::ANALYSIS as LOG_ANALYSIS;

impl Context {
    pub fn conflict_analysis(
        &mut self,
        clause_key: ClauseKey,
        config: &Config,
    ) -> Result<AnalysisResult, AnalysisError> {
        log::trace!(target: LOG_ANALYSIS, "Analysis called on {clause_key} at level {}", self.levels.index());
        if self.levels.index() == 0 {
            return Ok(AnalysisResult::FundamentalConflict(clause_key));
        }
        let Ok(conflict_clause) = self.clause_store.get(clause_key) else {
            panic!("x");
        };
        // log::trace!(target: LOG_ANALYSIS, "Clause {conflict_clause}");

        if let config::VSIDS::Chaff = config.vsids_variant {
            self.variables.apply_VSIDS(
                conflict_clause
                    .deref()
                    .iter()
                    .map(|literal| literal.index()),
                config,
            );
        }

        // this could be made persistent, but tying it to the solve may require a cell and lots of unsafe
        let mut the_buffer = ResolutionBuffer::from_variable_store(&self.variables);

        the_buffer.clear_literal(self.levels.top().choice());
        for (_, lit) in self.levels.top().observations() {
            the_buffer.clear_literal(*lit);
        }
        match the_buffer.set_inital_clause(conflict_clause, clause_key) {
            Ok(()) => {}
            Err(_) => return Err(AnalysisError::Buffer),
        };

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
                &mut self.traces,
                config,
            );
            match buffer_status {
                Ok(BufOk::Proof) => {}
                Ok(BufOk::FirstUIP) => {}
                Ok(BufOk::Exhausted) => {
                    if config.stopping_criteria == StoppingCriteria::FirstUIP {
                        return Err(AnalysisError::FailedStoppingCriteria);
                    }
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
                self.clause_store.bump_activity(*key as u32, config);
            }

            /*
            TODO: Alternative?
            Strengthening iterates through all the proven literals.
            This is skipped for a literal whose proof is to be noted
            This is also skipped for binary clauses, as if the other literal is proven the assertion will also be added as a proof, regardless
             */
            if the_buffer.clause_legnth() > 2 {
                the_buffer.strengthen_given(self.proven_literals());
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

            let output = true;

            match resolved_clause.len() {
                0 => Err(AnalysisError::EmptyResolution),
                1 => {
                    self.store_literal(
                        the_literal,
                        LiteralSource::Resolution(clause_key),
                        unsafe { the_buffer.take_trail() },
                    );

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
