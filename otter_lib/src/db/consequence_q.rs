use std::borrow::Borrow;

use crate::{
    context::Context,
    db::keys::ChoiceIndex,
    misc::log::targets::{self},
    structures::literal::{abLiteral, Literal},
    types::err::{self},
};

pub type ConsequenceQ = std::collections::VecDeque<(abLiteral, ChoiceIndex)>;

pub enum Ok {
    Qd,
}

impl Context {
    pub fn get_consequence(&mut self) -> Option<(abLiteral, ChoiceIndex)> {
        self.consequence_q.pop_front()
    }

    pub fn clear_consequences(&mut self, to: ChoiceIndex) {
        self.consequence_q.retain(|(_, c)| *c < to);
    }

    pub fn q_literal(&mut self, literal: impl Borrow<abLiteral>) -> Result<Ok, err::Queue> {
        let Ok(_) = self.atom_db.set_value(
            literal.borrow().atom(),
            literal.borrow().polarity(),
            Some(self.literal_db.choice_count()),
        ) else {
            log::trace!(target: targets::QUEUE, "Queueing {} failed.", literal.borrow());
            return Err(err::Queue::Conflict);
        };

        // TODO: improvements?
        self.consequence_q
            .push_back((*literal.borrow(), self.literal_db.choice_count()));

        Ok(Ok::Qd)
    }
}
