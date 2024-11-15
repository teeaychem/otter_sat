use std::borrow::Borrow;

use crate::{
    db::literal::{ChosenLiteral, LiteralDB},
    structures::literal::Literal,
    types::gen,
};

impl LiteralDB {
    pub(super) fn top_mut(&mut self) -> &mut ChosenLiteral {
        let last_choice_index = self.choice_stack.len() - 1;
        unsafe { self.choice_stack.get_unchecked_mut(last_choice_index) }
    }
}

impl ChosenLiteral {
    pub(super) fn new(literal: Literal) -> Self {
        Self {
            choice: literal,
            consequences: vec![],
        }
    }

    #[allow(dead_code)]
    pub fn consequences(&self) -> &[(gen::src::Literal, Literal)] {
        &self.consequences
    }

    pub(super) fn record_consequence(
        &mut self,
        literal: impl Borrow<Literal>,
        source: gen::src::Literal,
    ) {
        self.consequences.push((source, *literal.borrow()))
    }
}

#[allow(clippy::derivable_impls)]
impl Default for super::ProvenLiterals {
    fn default() -> Self {
        Self {
            observations: Vec::default(),
        }
    }
}

impl super::ProvenLiterals {
    pub fn record_literal(&mut self, literal: impl Borrow<Literal>) {
        self.observations.push(*literal.borrow())
    }
}
