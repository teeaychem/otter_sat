use crate::{
    context::Context,
    structures::{
        literal::{Literal, LiteralSource, LiteralTrait},
        variable::VariableId,
    },
};

use std::{collections::BTreeSet, ops::Deref};

use super::core::ContextFailure;

/// General order for pairs related to booleans is 0 is false, 1 is true
pub fn pure_choices<'l>(
    clauses: impl Iterator<Item = &'l [Literal]>,
) -> (Vec<VariableId>, Vec<VariableId>) {
    let mut the_true: BTreeSet<VariableId> = BTreeSet::new();
    let mut the_false: BTreeSet<VariableId> = BTreeSet::new();

    clauses.for_each(|literals| {
        literals.iter().for_each(|literal| {
            match literal.polarity() {
                true => the_true.insert(literal.v_id()),
                false => the_false.insert(literal.v_id()),
            };
        });
    });

    let pure_false: Vec<_> = the_false.difference(&the_true).copied().collect();
    let pure_true: Vec<_> = the_true.difference(&the_false).copied().collect();
    (pure_false, pure_true)
}

pub enum PreFailure {
    PureFailure,
}

impl Context {
    pub fn preprocess(&mut self) -> Result<(), PreFailure> {
        if self.config.preprocessing {
            match self.set_pure() {
                Ok(()) => {}
                Err(_) => {
                    log::error!(target: crate::log::targets::PREPROCESSING, "Failed to set pure literals");
                    return Err(PreFailure::PureFailure);
                }
            };
        }
        Ok(())
    }

    pub fn set_pure(&mut self) -> Result<(), ContextFailure> {
        let (f, t) = crate::context::preprocessing::pure_choices(
            self.clause_store.all_clauses().map(|sc| sc.deref()),
        );

        for v_id in f.into_iter().chain(t) {
            let the_literal = Literal::new(v_id, false);
            self.q_literal(the_literal, LiteralSource::Pure)?
        }
        Ok(())
    }
}
