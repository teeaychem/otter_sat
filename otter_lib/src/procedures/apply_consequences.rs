use crate::{
    context::Context,
    db::{dbStatus, ClauseKey},
    dispatch::{library::delta, Dispatch},
    procedures::analysis,
    structures::literal::{self, abLiteral},
    types::err,
};

pub enum Ok {
    Conflict,
    UnitClause(ClauseKey),
    AssertingClause(ClauseKey, abLiteral),
    Exhausted,
}

impl Context {
    /// Expand queued consequences:
    /// Performs an analysis on apparent conflict.
    pub fn apply_consequences(&mut self) -> Result<Ok, err::Context> {
        'expansion: while let Some((literal, _)) = self.consequence_q.pop_front() {
            match unsafe { self.bcp(literal) } {
                Ok(()) => {}
                Err(err::BCP::CorruptWatch) => return Err(err::Context::BCP),
                Err(err::BCP::Conflict(key)) => {
                    //
                    if !self.literal_db.choice_made() {
                        self.status = dbStatus::Inconsistent;

                        if let Some(dispatcher) = &self.dispatcher {
                            let delta = delta::AtomDB::Unsatisfiable(key);
                            dispatcher(Dispatch::Delta(delta::Delta::AtomDB(delta)));
                        }

                        return Ok(Ok::Conflict);
                    }

                    let analysis_result = self.conflict_analysis(&key)?;

                    match analysis_result {
                        analysis::Ok::FundamentalConflict => {
                            panic!("impossible");
                            // Analysis is only called when some decision has been made, for now
                        }

                        analysis::Ok::MissedImplication {
                            clause_key: key,
                            asserted_literal: literal,
                        } => {
                            let the_clause =
                                unsafe { self.clause_db.get_db_clause_unchecked(&key)? };

                            let index = self.backjump_level(the_clause)?;
                            self.backjump(index);

                            self.q_literal(literal)?;

                            if let Some(dispatcher) = &self.dispatcher {
                                let delta = delta::BCP::Instance {
                                    clause: key,
                                    literal,
                                };
                                dispatcher(Dispatch::Delta(delta::Delta::BCP(delta)));
                            }
                            self.record_literal(literal, literal::Source::BCP(key));

                            continue 'expansion;
                        }

                        analysis::Ok::UnitClause(key) => {
                            return Ok(Ok::UnitClause(key));
                        }

                        analysis::Ok::AssertingClause {
                            clause_key: key,
                            asserted_literal: literal,
                        } => {
                            return Ok(Ok::AssertingClause(key, literal));
                        }
                    }
                }
            }
        }
        Ok(Ok::Exhausted)
    }
}
