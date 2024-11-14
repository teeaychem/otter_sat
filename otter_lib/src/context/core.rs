use std::borrow::Borrow;

use crate::{
    context::Context,
    db::keys::ClauseKey,
    dispatch::{
        report::{self},
        Dispatch,
    },
    structures::{literal::Literal, variable::Variable},
    types::{clause::ClauseSource, err, gen},
};

impl Context {
    /// Stores a clause with an automatically generated id.
    /// In order to use the clause the watch literals of the struct must be initialised.
    pub fn store_clause(
        &mut self,
        clause: Vec<Literal>,
        source: ClauseSource,
    ) -> Result<ClauseKey, err::ClauseDB> {
        self.clause_db
            .insert_clause(source, clause, &mut self.variable_db)
    }

    pub fn note_literal<L: Borrow<Literal>>(&mut self, literal: L, source: gen::LiteralSource) {
        log::trace!("Noted {}", literal.borrow());
        self.literal_db.record_literal(*literal.borrow(), source);
    }

    pub fn valuation_string(&self) -> String {
        self.variable_db
            .valuation()
            .iter()
            .enumerate()
            .filter_map(|(i, v)| {
                let idx = i as Variable;
                match v {
                    None => None,
                    Some(true) => Some(format!(
                        " {}",
                        self.variable_db.external_representation(idx)
                    )),
                    Some(false) => Some(format!(
                        "-{}",
                        self.variable_db.external_representation(idx)
                    )),
                }
            })
            .collect::<Vec<_>>()
            .join(" ")
    }

    pub fn internal_valuation_string(&self) -> String {
        let mut v = self
            .variable_db
            .valuation()
            .iter()
            .enumerate()
            .filter_map(|(i, v)| match v {
                None => None,
                Some(true) => Some(i as isize),
                Some(false) => Some(-(i as isize)),
            })
            .collect::<Vec<_>>();
        v.sort_unstable();
        v.iter()
            .map(|v| v.to_string())
            .collect::<Vec<_>>()
            .join(" ")
    }

    pub fn report_active(&self) {
        self.clause_db.report_active();

        for literal in self.literal_db.proven_literals() {
            let report = report::VariableDB::Active(*literal);
            self.tx.send(Dispatch::VariableDBReport(report));
        }
    }
}
