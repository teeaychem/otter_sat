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
    dispatch::{library::delta, Dispatch},
    structures::{clause::Clause, literal::Literal},
    types::{err, gen},
};

impl Context {
    pub fn record_literal(&mut self, literal: impl Borrow<Literal>, source: gen::src::Literal) {
        match source {
            gen::src::Literal::Choice => {}

            gen::src::Literal::Original => {
                if let Some(dispatcher) = &self.dispatcher {
                    let delta = delta::LiteralDB::Original(*literal.borrow());
                    dispatcher(Dispatch::Delta(delta::Delta::LiteralDB(delta)));
                }
                self.clause_db.unit.push(*literal.borrow())
            }

            gen::src::Literal::BCP(_) => match self.literal_db.choice_stack.len() {
                0 => {
                    if let Some(dispatcher) = &self.dispatcher {
                        let delta = delta::LiteralDB::ProofBCP(*literal.borrow());
                        dispatcher(Dispatch::Delta(delta::Delta::LiteralDB(delta)));
                    }
                    self.clause_db.unit.push(*literal.borrow())
                }
                _ => self
                    .literal_db
                    .top_mut()
                    .record_consequence(literal, source),
            },

            gen::src::Literal::Resolution => {
                // Resoluion implies deduction via (known) clauses
                if let Some(dispatcher) = &self.dispatcher {
                    let delta = delta::LiteralDB::ProofResolution(*literal.borrow());
                    dispatcher(Dispatch::Delta(delta::Delta::LiteralDB(delta)));
                }
                self.clause_db.unit.push(*literal.borrow())
            }
        }
    }

    pub fn record_clause(
        &mut self,
        clause: Clause,
        source: gen::src::Clause,
    ) -> Result<ClauseKey, err::ClauseDB> {
        match clause.len() {
            0 => Err(err::ClauseDB::EmptyClause),

            1 => {
                let literal = unsafe { clause.get_unchecked(0) };
                self.add_literal(literal);
                Ok(ClauseKey::Unit(*literal))
            }

            _ => self.clause_db.store(clause, source, &mut self.variable_db),
        }
    }
}
