use crate::{
    config::{self, StoppingCriteria},
    context::Context,
    db::keys::ClauseKey,
    misc::log::targets::{self},
    structures::clause::Clause,
    transient::resolution_buffer::ResolutionBuffer,
    types::{
        err::{self},
        gen::{self},
    },
};

impl Context {
    pub fn conflict_analysis(&mut self, key: ClauseKey) -> Result<gen::Analysis, err::Analysis> {
        log::trace!(target: targets::ANALYSIS, "Analysis of {key} at level {}", self.literal_db.choice_count());

        if let config::VSIDS::Chaff = self.config.vsids_variant {
            self.atom_db
                .apply_VSIDS(self.clause_db.get_db_clause(key)?.atoms());
        }

        let mut the_buffer =
            ResolutionBuffer::from_atom_db(&self.atom_db, self.dispatcher.clone(), &self.config);

        the_buffer.clear_literal(self.literal_db.last_choice());
        for (_, lit) in self.literal_db.last_consequences() {
            the_buffer.clear_literal(*lit);
        }

        match the_buffer.resolve_with(
            key,
            &self.literal_db,
            &mut self.clause_db,
            &mut self.atom_db,
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
            self.atom_db.apply_VSIDS(the_buffer.atoms_used());
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

        let the_literal = match assertion_index {
            None => {
                log::error!(target: targets::ANALYSIS, "Failed to resolve to an asserting clause");
                return Err(err::Analysis::NoAssertion);
            }
            Some(index) => *unsafe { resolved_clause.get_unchecked(index) },
        };

        match resolved_clause.len() {
            0 => Err(err::Analysis::EmptyResolution),
            1 => {
                let key = self.record_clause(the_literal, gen::src::Clause::Resolution)?;
                Ok(gen::Analysis::UnitClause(key))
            }
            _ => {
                let key = self.record_clause(resolved_clause, gen::src::Clause::Resolution)?;
                Ok(gen::Analysis::AssertingClause(key, the_literal))
            }
        }
    }
}
