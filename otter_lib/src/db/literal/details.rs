use std::borrow::Borrow;

use crate::{
    db::literal::{ChosenLiteral, LiteralDB},
    structures::literal::abLiteral,
    types::gen,
};

impl LiteralDB {
    pub fn top_mut(&mut self) -> &mut ChosenLiteral {
        let last_choice_index = self.choice_stack.len() - 1;
        unsafe { self.choice_stack.get_unchecked_mut(last_choice_index) }
    }
}

impl ChosenLiteral {
    pub(super) fn new(literal: abLiteral) -> Self {
        Self {
            choice: literal,
            consequences: vec![],
        }
    }

    #[allow(dead_code)]
    pub fn consequences(&self) -> &[(gen::src::Literal, abLiteral)] {
        &self.consequences
    }

    pub fn record_consequence(
        &mut self,
        literal: impl Borrow<abLiteral>,
        source: gen::src::Literal,
    ) {
        self.consequences.push((source, *literal.borrow()))
    }
}
