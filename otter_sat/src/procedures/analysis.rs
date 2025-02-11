/*!
Analysis of an unsatisfiable clause.

Takes a key to a clause which is unsatisfiable on the current valuation and returns an asserting clause.

In other words, conflict analysis takes a key to a clause which is unsatisfiable on the current valuation and applies resolution using the clauses used to (eventually) make the observation of a conflict given decisions made.

For details on resolution, see the [resolution buffer](crate::transient::resolution_buffer).

For the method, see: [conflict_analysis](GenericContext::conflict_analysis).

# Example

```rust, ignore
let analysis_result = self.conflict_analysis(&key)?;

match analysis_result {
    analysis::ConflictAnalysisOk::FundamentalConflict => {
        ...
    }

    analysis::ConflictAnalysisOk::RepeatImplication {
        clause_key: key,
        asserted_literal: literal,
    } => {
        Ok(AssertingClause(key, literal))
    }

    analysis::ConflictAnalysisOk::UnitClause(key) => {
        Ok(UnitClause(key))
    }

    analysis::ConflictAnalysisOk::AssertingClause {
        clause_key: key,
        asserted_literal: literal,
    } => {
        Ok(AssertingClause(key, literal))
    }
}
```
*/

use crate::{
    config::StoppingCriteria,
    context::GenericContext,
    db::ClauseKey,
    misc::log::targets::{self},
    structures::{
        clause::{Clause, ClauseSource},
        literal::{CLiteral, Literal},
        valuation::Valuation,
    },
    transient::resolution_buffer::{self, ResolutionBuffer},
    types::err::{self},
};

/// Possible 'Ok' results from conflict analysis.
pub enum ConflictAnalysisOk {
    /// The conflict clause was asserting at some previous decision level.
    MissedPropagation { key: ClauseKey, literal: CLiteral },

    /// The result of analysis is a unit clause.
    UnitClause { key: CLiteral },

    /// A fundamental conflict is identified, and so the current formula is unsatisfiable.
    ///
    /// Note, this result is unused, at present.
    /// For, conflict analysis is only called after a decision has been made, and so in case of conflict a clause asserting the negation of some decision will always be available (as the decision must have appeared in some clause to derive a conflict).
    FundamentalConflict,

    /// The result of analysis is a (non-unit) asserting clause.
    AssertingClause { key: ClauseKey, literal: CLiteral },
}

impl<R: rand::Rng + std::default::Default> GenericContext<R> {
    /// For details on conflict analysis see the [analysis](crate::procedures::analysis) procedure.
    pub fn conflict_analysis(
        &mut self,
        key: &ClauseKey,
    ) -> Result<ConflictAnalysisOk, err::ErrorKind> {
        log::trace!(target: targets::ANALYSIS, "Analysis of {key} at level {}", self.literal_db.current_level());

        if let crate::config::vsids::VSIDS::Chaff = self.config.vsids_variant {
            self.atom_db
                .bump_relative(unsafe { self.clause_db.get_unchecked(key)?.atoms() });
        }

        let mut backstep_valuation = self.atom_db.valuation_canonical().clone();
        unsafe {
            backstep_valuation.clear_value_of(self.literal_db.top_decision_unchecked().atom());
            for assertion in self.literal_db.top_consequences_unchecked() {
                backstep_valuation.clear_value_of(assertion.atom());
            }
        }

        // TODO: As the previous valuation is stored, it'd make sense to use that instead of rolling back the current valuation.
        let mut the_buffer = ResolutionBuffer::from_valuation(
            &backstep_valuation,
            self.dispatcher.clone(),
            &self.config,
        );

        // Some decision must have been made for conflict analysis to take place.

        match the_buffer.resolve_through_current_level(
            key,
            &self.literal_db,
            &mut self.clause_db,
            &mut self.atom_db,
        ) {
            Ok(resolution_buffer::ResolutionOk::UnitClause)
            | Ok(resolution_buffer::ResolutionOk::FirstUIP) => {}
            Ok(resolution_buffer::ResolutionOk::Exhausted) => {
                if self.config.stopping_criteria == StoppingCriteria::FirstUIP {
                    log::error!(target: targets::ANALYSIS, "Wrong stopping criteria.");
                    return Err(err::ErrorKind::from(
                        err::AnalysisError::FailedStoppingCriteria,
                    ));
                }
            }
            Ok(resolution_buffer::ResolutionOk::Repeat(k, l)) => {
                return Ok(ConflictAnalysisOk::MissedPropagation { key: k, literal: l });
            }
            Err(buffer_error) => {
                return Err(err::ErrorKind::ResolutionBuffer(buffer_error));
            }
        }

        if let crate::config::vsids::VSIDS::MiniSAT = self.config.vsids_variant {
            self.atom_db.bump_relative(the_buffer.atoms_used());
        }

        /*
        TODO: Alternative? Re-enable?
        Strengthening iterates through all the proven literals.
        This is skipped for a literal whose proof is to be noted.
        This is also skipped for binary clauses, as if the other literal is proven the assertion will also be added as a proof, regardless.
         */
        // if the_buffer.clause_legnth() > 2 {
        //     the_buffer.strengthen_given(self.clause_db.all_unit_clauses());
        // }

        let premises = the_buffer.take_premises();
        let (resolved_clause, assertion_index) = the_buffer.to_assertion_clause();

        let literal = match assertion_index {
            None => {
                log::error!(target: targets::ANALYSIS, "Failed to resolve to an asserting clause");
                return Err(err::ErrorKind::from(err::AnalysisError::NoAssertion));
            }
            // Safe, by operation of the resolution buffer.
            Some(index) => *unsafe { resolved_clause.get_unchecked(index) },
        };

        match resolved_clause.len() {
            0 => Err(err::ErrorKind::from(err::AnalysisError::EmptyResolution)),
            1 => {
                self.backjump(self.literal_db.lowest_decision_level());
                self.clause_db.store(
                    literal,
                    ClauseSource::Resolution,
                    &mut self.atom_db,
                    None,
                    premises,
                )?;
                Ok(ConflictAnalysisOk::UnitClause { key: literal })
            }
            _ => {
                let index = self.non_chronological_backjump_level(&resolved_clause)?;

                self.backjump(index);

                let key = self.clause_db.store(
                    resolved_clause,
                    ClauseSource::Resolution,
                    &mut self.atom_db,
                    None,
                    premises,
                )?;
                Ok(ConflictAnalysisOk::AssertingClause { key, literal })
            }
        }
    }
}
