use std::borrow::Borrow;

use crate::{
    context::Context,
    db::keys::ChoiceIndex,
    structures::literal::{Literal, LiteralT},
    types::{err, gen},
};

pub type ConsequenceQ = std::collections::VecDeque<(Literal, ChoiceIndex)>;

impl Context {
    pub fn get_consequence(&mut self) -> Option<(Literal, ChoiceIndex)> {
        self.consequence_q.pop_front()
    }

    pub fn clear_consequences(&mut self, to: ChoiceIndex) {
        self.consequence_q.retain(|(_, c)| *c < to);
    }

    pub fn q_literal<L: Borrow<Literal>>(
        &mut self,
        literal: L,
    ) -> Result<gen::QStatus, err::Context> {
        let Ok(_) = self.variable_db.set_value(
            literal.borrow().var(),
            literal.borrow().polarity(),
            Some(self.literal_db.choice_count()),
        ) else {
            return Err(err::Context::QueueConflict);
        };

        // TODO: improve push back consequence
        self.consequence_q
            .push_back((*literal.borrow(), self.literal_db.choice_count()));

        Ok(gen::QStatus::Qd)
    }
}
