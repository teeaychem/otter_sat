pub mod clause;
pub mod consequence_q;
pub mod keys;
pub mod literal;
pub mod variable;

use std::borrow::Borrow;

use keys::ClauseKey;

use crate::{
    context::Context,
    structures::literal::Literal,
    types::{
        err::{self},
        gen::{self},
    },
};

impl Context {
    /// Stores a clause with an automatically generated id.
    /// In order to use the clause the watch literals of the struct must be initialised.
    pub fn store_clause(
        &mut self,
        clause: Vec<Literal>,
        source: gen::src::Clause,
    ) -> Result<ClauseKey, err::ClauseDB> {
        self.clause_db
            .insert_clause(source, clause, &mut self.variable_db)
    }

    pub fn note_literal(&mut self, literal: impl Borrow<Literal>, source: gen::src::Literal) {
        log::trace!("Noted {}", literal.borrow());
        self.literal_db.record_literal(*literal.borrow(), source);
    }
}
