use std::borrow::Borrow;

use crate::{
    db::literal::{ChosenLiteral, LiteralDB},
    structures::literal::{self, abLiteral},
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
    pub fn consequences(&self) -> &[(literal::Source, abLiteral)] {
        &self.consequences
    }

    pub fn record_consequence(&mut self, literal: impl Borrow<abLiteral>, source: literal::Source) {
        self.consequences.push((source, *literal.borrow()))
    }
}
