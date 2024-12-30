//! Bases for holding data relevant to a solve.

pub mod atom;
pub mod clause;
pub mod consequence_q;
pub mod keys;
pub mod literal;

use std::borrow::Borrow;

use keys::ClauseKey;

use crate::{
    context::Context,
    dispatch::{
        library::delta::{self, Delta},
        Dispatch,
    },
    structures::{
        clause::{Clause, Source as ClauseSource},
        literal::{abLiteral, Source as LiteralSource},
    },
    types::err,
};

#[allow(non_camel_case_types)]
#[derive(Debug, PartialEq, Eq)]
pub enum dbStatus {
    Consistent,
    Inconsistent,
    Unknown,
}

impl Context {
    pub fn record_literal(&mut self, literal: impl Borrow<abLiteral>, source: LiteralSource) {
        match source {
            LiteralSource::Choice => {}

            LiteralSource::BCP(_) => match self.literal_db.choice_stack.len() {
                0 => {
                    if let Some(dispatcher) = &self.dispatcher {
                        let delta = delta::ClauseDB::BCP(ClauseKey::Unit(*literal.borrow()));
                        dispatcher(Dispatch::Delta(delta::Delta::ClauseDB(delta)));
                    }
                    self.clause_db.unit.push(*literal.borrow())
                }
                _ => self
                    .literal_db
                    .top_mut()
                    .record_consequence(literal, source),
            },
        }
    }

    /// Makes a record of a clause.
    ///
    /// The details are handled by the clause database.
    /// Dispatches regarding the clause are made here.
    pub fn record_clause(
        &mut self,
        clause: impl Clause,
        source: ClauseSource,
    ) -> Result<ClauseKey, err::ClauseDB> {
        let the_key = self.clause_db.store(clause, source, &mut self.atom_db)?;

        if let Some(dispatcher) = &self.dispatcher {
            match the_key {
                ClauseKey::Unit(_) => match source {
                    ClauseSource::Resolution => {
                        let delta = delta::ClauseDB::Added(the_key);
                        dispatcher(Dispatch::Delta(delta::Delta::ClauseDB(delta)));
                    }

                    ClauseSource::Original => {
                        let delta = delta::ClauseDB::Original(the_key);
                        dispatcher(Dispatch::Delta(delta::Delta::ClauseDB(delta)));
                    }
                },

                _ => {
                    let db_clause = self.clause_db.get_db_clause(the_key)?;
                    match db_clause.size() {
                        0 => panic!("impossible"),
                        1 => {}
                        _ => {
                            let delta = delta::ClauseDB::ClauseStart;
                            dispatcher(Dispatch::Delta(Delta::ClauseDB(delta)));
                            for literal in db_clause.literals() {
                                let delta = delta::ClauseDB::ClauseLiteral(*literal);
                                dispatcher(Dispatch::Delta(Delta::ClauseDB(delta)));
                            }
                            let delta = {
                                match source {
                                    ClauseSource::Original => delta::ClauseDB::Original(the_key),
                                    ClauseSource::Resolution => delta::ClauseDB::Added(the_key),
                                }
                            };
                            dispatcher(Dispatch::Delta(Delta::ClauseDB(delta)));
                        }
                    }
                }
            }
        }

        Ok(the_key)
    }
}
