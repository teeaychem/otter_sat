use crate::{
    config::{self, StoppingCriteria},
    context::Context,
    db::keys::ClauseKey,
    misc::log::targets::{self},
    structures::clause::ClauseT,
    transient::resolution_buffer::ResolutionBuffer,
    types::{
        err::{self},
        gen::{self},
    },
};

#[allow(unused_imports)]

impl Context {
    pub fn conflict_analysis(&mut self, key: ClauseKey) -> Result<gen::Analysis, err::Analysis> {
        log::trace!(target: targets::ANALYSIS, "Analysis of {key} at level {}", self.literal_db.choice_count());

        if let config::VSIDS::Chaff = self.config.vsids_variant {
            self.variable_db
                .apply_VSIDS(self.clause_db.get(key)?.variables());
        }

        let mut the_buffer = ResolutionBuffer::from_variable_store(
            &self.variable_db,
            self.dispatcher.clone(),
            &self.config,
        );

        the_buffer.clear_literal(self.literal_db.last_choice());
        for (_, lit) in self.literal_db.last_consequences() {
            the_buffer.clear_literal(*lit);
        }

        match the_buffer.resolve_with(
            key,
            &self.literal_db,
            &mut self.clause_db,
            &mut self.variable_db,
        ) {
            Ok(gen::RBuf::Proof) | Ok(gen::RBuf::FirstUIP) => {}
            Ok(gen::RBuf::Exhausted) => {
                if self.config.stopping_criteria == StoppingCriteria::FirstUIP {
                    log::error!(target: targets::ANALYSIS, "Wrong stopping criteria.");
                    return Err(err::Analysis::FailedStoppingCriteria);
                }
            }
            Ok(gen::RBuf::Missed(k, l)) => {
                return Ok(gen::Analysis::MissedImplication(k, l));
            }
            Err(_buffer_error) => {
                return Err(err::Analysis::Buffer);
            }
        }

        if let config::VSIDS::MiniSAT = self.config.vsids_variant {
            self.variable_db.apply_VSIDS(the_buffer.variables_used());
        }

        /*
        TODO: Alternative?
        Strengthening iterates through all the proven literals.
        This is skipped for a literal whose proof is to be noted
        This is also skipped for binary clauses, as if the other literal is proven the assertion will also be added as a proof, regardless
         */
        if the_buffer.clause_legnth() > 2 {
            the_buffer.strengthen_given(self.literal_db.proven_literals().iter());
        }

        let (asserted_literal, mut resolved_clause) = the_buffer.to_assertion_clause();
        // TODO: Revise this, maybe, as it means the watch is in the last place looked…
        if let Some(assertion) = asserted_literal {
            resolved_clause.push(assertion);
        }

        let the_literal = match asserted_literal {
            None => {
                log::error!(target: targets::ANALYSIS, "Failed to resolve to an asserting clause");
                return Err(err::Analysis::NoAssertion);
            }
            Some(literal) => literal,
        };

        match resolved_clause.len() {
            0 => Err(err::Analysis::EmptyResolution),
            1 => Ok(gen::Analysis::Proof(key, the_literal)),
            _ => {
                let key = self.clause_db.store(
                    resolved_clause,
                    gen::src::Clause::Resolution,
                    &mut self.variable_db,
                )?;

                Ok(gen::Analysis::AssertingClause(key, the_literal))
            }
        }
    }
}
