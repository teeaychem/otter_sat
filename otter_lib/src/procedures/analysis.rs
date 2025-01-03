use crate::{
    config::StoppingCriteria,
    context::GenericContext,
    db::ClauseKey,
    misc::log::targets::{self},
    structures::{
        clause::{self, Clause},
        literal::{abLiteral, Literal},
    },
    transient::resolution_buffer::{self, ResolutionBuffer},
    types::err::{self},
};

/// Possible 'Ok' results from conflict analysis.
pub enum Ok {
    /// The conflict clause was asserting at some previous decision level.
    MissedImplication {
        clause_key: ClauseKey,
        asserted_literal: abLiteral,
    },

    /// The result of analysis is a unit clause.
    UnitClause(ClauseKey),

    /// A fundamental conflict is identified, and so the current formula is unsatisfiable.
    FundamentalConflict,

    /// The result of analysis is a (non-unit) asserting clause.
    AssertingClause {
        clause_key: ClauseKey,
        asserted_literal: abLiteral,
    },
}

impl<R: rand::Rng + std::default::Default> GenericContext<R> {
    pub fn conflict_analysis(&mut self, key: &ClauseKey) -> Result<Ok, err::Analysis> {
        log::trace!(target: targets::ANALYSIS, "Analysis of {key} at level {}", self.literal_db.choice_count());

        if let crate::config::vsids::VSIDS::Chaff = self.config.vsids_variant {
            self.atom_db
                .bump_relative(unsafe { self.clause_db.get_db_clause_unchecked(key)?.atoms() });
        }

        // TODO: As the previous valuation is stored, it'd make sense to use that instead of rolling back the current valuation.
        let mut the_buffer = ResolutionBuffer::from_valuation(
            self.atom_db.valuation(),
            self.dispatcher.clone(),
            &self.config,
        );

        the_buffer.clear_atom_value(unsafe { self.literal_db.last_choice_unchecked().atom() });
        for (_, literal) in self.literal_db.last_consequences_unchecked() {
            the_buffer.clear_atom_value(literal.atom());
        }

        match the_buffer.resolve_through_current_level(
            key,
            &self.literal_db,
            &mut self.clause_db,
            &mut self.atom_db,
        ) {
            Ok(resolution_buffer::Ok::UnitClause) | Ok(resolution_buffer::Ok::FirstUIP) => {}
            Ok(resolution_buffer::Ok::Exhausted) => {
                if self.config.stopping_criteria == StoppingCriteria::FirstUIP {
                    log::error!(target: targets::ANALYSIS, "Wrong stopping criteria.");
                    return Err(err::Analysis::FailedStoppingCriteria);
                }
            }
            Ok(resolution_buffer::Ok::Missed(k, l)) => {
                return Ok(Ok::MissedImplication {
                    clause_key: k,
                    asserted_literal: l,
                });
            }
            Err(_buffer_error) => {
                return Err(err::Analysis::Buffer);
            }
        }

        if let crate::config::vsids::VSIDS::MiniSAT = self.config.vsids_variant {
            self.atom_db.bump_relative(the_buffer.atoms_used());
        }

        /*
        TODO: Alternative?
        Strengthening iterates through all the proven literals.
        This is skipped for a literal whose proof is to be noted
        This is also skipped for binary clauses, as if the other literal is proven the assertion will also be added as a proof, regardless
         */
        if the_buffer.clause_legnth() > 2 {
            the_buffer.strengthen_given(self.clause_db.all_unit_clauses());
        }

        let (resolved_clause, assertion_index) = the_buffer.to_assertion_clause();

        let asserted_literal = match assertion_index {
            None => {
                log::error!(target: targets::ANALYSIS, "Failed to resolve to an asserting clause");
                return Err(err::Analysis::NoAssertion);
            }
            Some(index) => *unsafe { resolved_clause.get_unchecked(index) },
        };

        match resolved_clause.len() {
            0 => Err(err::Analysis::EmptyResolution),
            1 => {
                let key = self.record_clause(asserted_literal, clause::Source::Resolution)?;
                Ok(Ok::UnitClause(key))
            }
            _ => {
                let key = self.record_clause(resolved_clause, clause::Source::Resolution)?;
                Ok(Ok::AssertingClause {
                    clause_key: key,
                    asserted_literal,
                })
            }
        }
    }
}
