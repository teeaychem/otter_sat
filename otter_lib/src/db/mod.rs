//! Bases for holding data relevant to a solve.

pub mod clause;
pub mod consequence_q;
pub mod keys;
pub mod literal;
pub mod variable;

use std::borrow::Borrow;

use keys::ClauseKey;

use crate::{
    context::Context,
    dispatch::{
        library::delta::{self, Delta},
        Dispatch,
    },
    structures::{
        clause::{Clause, ClauseT},
        literal::Literal,
    },
    types::{err, gen},
};

impl Context {
    pub fn record_literal(&mut self, literal: impl Borrow<Literal>, source: gen::src::Literal) {
        match source {
            gen::src::Literal::Choice => {}

            gen::src::Literal::BCP(_) => match self.literal_db.choice_stack.len() {
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
        clause: impl ClauseT,
        source: gen::src::Clause,
    ) -> Result<ClauseKey, err::ClauseDB> {
        let the_key = self
            .clause_db
            .store(clause, source, &mut self.variable_db)?;

        if let Some(dispatcher) = &self.dispatcher {
            match the_key {
                ClauseKey::Unit(_) => match source {
                    gen::src::Clause::Resolution => {
                        let delta = delta::ClauseDB::Added(the_key);
                        dispatcher(Dispatch::Delta(delta::Delta::ClauseDB(delta)));
                    }

                    gen::src::Clause::Original => {
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
                                    gen::src::Clause::Original => {
                                        delta::ClauseDB::Original(the_key)
                                    }
                                    gen::src::Clause::Resolution => {
                                        delta::ClauseDB::Added(the_key)
                                    }
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
