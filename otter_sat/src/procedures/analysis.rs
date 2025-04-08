/*!
Analysis of an unsatisfiable clause.

Takes a key to a clause which is unsatisfiable on the current valuation and returns an asserting clause.

In other words, conflict analysis takes a key to a clause which is unsatisfiable on the current valuation and applies resolution using the clauses used to (eventually) make the observation of a conflict given decisions made.

For details on resolution, see the [atom cells](crate::atom_cells) structure.

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
    atom_cells::ResolutionOk,
    config,
    context::GenericContext,
    db::ClauseKey,
    misc::log::targets::{self},
    structures::{
        clause::{Clause, ClauseSource},
        literal::{CLiteral, Literal},
    },
    types::err::{self},
};

/// Possible 'Ok' results from conflict analysis.
pub enum AnalysisResult {
    /// The conflict clause was asserting at some previous decision level.
    MissedPropagation {
        /// The key to the clause.
        key: ClauseKey,
        /// The literal asserted by the clause.
        literal: CLiteral,
    },

    /// The result of analysis is a unit clause.
    UnitClause {
        /// The literal of the clause.
        literal: CLiteral,
    },

    /// A fundamental conflict is identified, and so the current formula is unsatisfiable.
    ///
    /// Note, this result is unused, at present.
    /// For, conflict analysis is only called after a decision has been made, and so in case of conflict a clause asserting the negation of some decision will always be available (as the decision must have appeared in some clause to derive a conflict).
    FundamentalConflict,

    /// The result of analysis is a (non-unit) asserting clause.
    AssertingClause {
        /// The key of the asserting clause.
        key: ClauseKey,

        /// The literal asserted by the clause.
        literal: CLiteral,
    },
}

impl<R: rand::Rng + std::default::Default> GenericContext<R> {
    /// For details on conflict analysis see the [analysis](crate::procedures::analysis) procedure.
    pub fn conflict_analysis(&mut self, key: &ClauseKey) -> Result<AnalysisResult, err::ErrorKind> {
        log::info!(target: targets::ANALYSIS, "Analysis of {key} at level {}", self.trail.level());
        log::info!(target: targets::ANALYSIS, "Level: {:?}", self.trail.top_level_assignments());
        log::info!(target: targets::ANALYSIS, "Valuation: {}", self.valuation_string());

        if let config::vsids::VSIDS::Chaff = self.config.vsids.value {
            crate::db::atom::activity::bump_relative(
                unsafe { self.clause_db.get_unchecked(key).atoms() },
                &mut self.atom_activity,
                &mut self.config.atom_bump,
                &mut self.config.atom_decay,
            );
        }

        self.atom_cells.refresh();
        // Safety: Some decision must have been made for conflict analysis to take place.

        for literal in self.trail.top_level_assignments() {
            self.atom_cells.mark_backjump(literal.atom());
        }

        match self.atom_cells.resolve_through_current_level(
            key,
            &self.valuation,
            &mut self.clause_db,
            &mut self.watches,
            &mut self.trail,
        ) {
            Ok(ResolutionOk::UnitClause) | Ok(ResolutionOk::UIP) => {}

            Ok(ResolutionOk::Repeat(key, literal)) => {
                return Ok(AnalysisResult::MissedPropagation { key, literal });
            }

            Err(buffer_error) => {
                return Err(err::ErrorKind::ResolutionBuffer(buffer_error));
            }
        }

        if let config::vsids::VSIDS::MiniSAT = self.config.vsids.value {
            crate::db::atom::activity::bump_relative(
                self.atom_cells.atoms_used(),
                &mut self.atom_activity,
                &mut self.config.atom_bump,
                &mut self.config.atom_decay,
            );
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

        let premises = self.atom_cells.take_premises();
        let clause = self.atom_cells.to_assertion_clause();
        let literal = *unsafe { clause.get_unchecked(0) };
        log::info!(target: targets::ANALYSIS, "Addition clause: {:?}", clause);

        match clause.len() {
            0 => Err(err::ErrorKind::from(err::AnalysisError::EmptyResolution)),

            1 => {
                self.backjump(self.trail.lowest_decision_level());

                self.clause_db.store(
                    literal,
                    ClauseSource::Resolution,
                    &self.valuation,
                    &mut self.atom_cells,
                    &mut self.watches,
                    premises,
                )?;

                Ok(AnalysisResult::UnitClause { literal })
            }

            _ => {
                let index = self.non_chronological_backjump_level(&clause)?;

                self.backjump(index);

                let key = self.clause_db.store(
                    clause,
                    ClauseSource::Resolution,
                    &self.valuation,
                    &mut self.atom_cells,
                    &mut self.watches,
                    premises,
                )?;

                Ok(AnalysisResult::AssertingClause { key, literal })
            }
        }
    }
}
