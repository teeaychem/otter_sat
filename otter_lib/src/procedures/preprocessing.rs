use crate::{
    context::Context,
    misc::log::targets::{self},
    structures::{
        clause::Clause,
        literal::{Literal, LiteralT},
        variable::Variable,
    },
    types::{
        err::{self},
        gen::{self},
    },
};

use std::collections::BTreeSet;

/// General order for pairs related to booleans is 0 is false, 1 is true
pub fn pure_choices<'l>(
    clauses: impl Iterator<Item = &'l [Literal]>,
) -> (Vec<Variable>, Vec<Variable>) {
    let mut the_true: BTreeSet<Variable> = BTreeSet::new();
    let mut the_false: BTreeSet<Variable> = BTreeSet::new();

    clauses.for_each(|literals| {
        literals.iter().for_each(|literal| {
            match literal.polarity() {
                true => the_true.insert(literal.var()),
                false => the_false.insert(literal.var()),
            };
        });
    });

    let pure_false: Vec<_> = the_false.difference(&the_true).copied().collect();
    let pure_true: Vec<_> = the_true.difference(&the_false).copied().collect();
    (pure_false, pure_true)
}

impl Context {
    pub fn preprocess(&mut self) -> Result<(), err::Preprocessing> {
        if self.config.preprocessing {
            match self.set_pure() {
                Ok(()) => {}
                Err(_) => {
                    log::error!(target: targets::PREPROCESSING, "Failed to set pure literals");
                    return Err(err::Preprocessing::Pure);
                }
            };
        }
        Ok(())
    }

    pub fn set_pure(&mut self) -> Result<(), err::Queue> {
        let (f, t) = crate::procedures::preprocessing::pure_choices(
            self.clause_db.all_clauses().map(|sc| sc.literals()),
        );

        for v_id in f.into_iter().chain(t) {
            let the_literal = Literal::new(v_id, false);
            match self.q_literal(the_literal) {
                Ok(gen::Queue::Qd) => {
                    self.note_literal(the_literal, gen::src::Literal::Pure);
                }
                Err(e) => return Err(e),
            }
        }
        Ok(())
    }
}
