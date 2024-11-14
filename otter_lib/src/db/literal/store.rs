use crate::{
    db::{
        keys::ChoiceIndex,
        literal::{ChosenLiteral, LiteralDB},
    },
    dispatch::{
        delta::{self},
        Dispatch,
    },
    structures::literal::Literal,
    types::gen,
};

impl LiteralDB {
    pub fn make_choice(&mut self, choice: Literal) {
        self.chosen.push(ChosenLiteral::new(choice));
    }

    /*
    A recorded literal may be the consequence of a choice or `proven`.
    In some cases this is simple to determine when the record happens.
    For example, if a literal was an (external) assumption it is `proven`.
    Still, in some cases it's easier to check when recording the literal.
    So, checks are made here.
    */
    pub(crate) fn record_literal(&mut self, literal: Literal, source: gen::LiteralSource) {
        match source {
            gen::LiteralSource::Choice => {}
            gen::LiteralSource::Assumption => {
                let delta = delta::Level::Assumption(literal);
                self.tx.send(Dispatch::Level(delta));
                self.proven.record_literal(literal)
            }
            gen::LiteralSource::Pure => {
                let delta = delta::Level::Pure(literal);
                self.tx.send(Dispatch::Level(delta));
                self.proven.record_literal(literal)
            }
            gen::LiteralSource::BCP(_) => match self.chosen.len() {
                0 => {
                    let delta = delta::Level::BCP(literal);
                    self.tx.send(Dispatch::Level(delta));
                    self.proven.record_literal(literal)
                }
                _ => self.top_mut().record_consequence(literal, source),
            },
            gen::LiteralSource::Resolution(_) => {
                // Resoluion implies deduction via (known) clauses
                let delta = delta::Level::ResolutionProof(literal);
                self.tx.send(Dispatch::Level(delta));
                self.proven.record_literal(literal)
            }
            gen::LiteralSource::Analysis(_) => match self.chosen.len() {
                0 => self.proven.record_literal(literal),
                _ => self.top_mut().record_consequence(literal, source),
            },
            gen::LiteralSource::Missed(_) => match self.chosen.len() {
                0 => self.proven.record_literal(literal),
                _ => self.top_mut().record_consequence(literal, source),
            },
        }
    }

    pub fn current_choice(&self) -> Literal {
        unsafe { self.chosen.get_unchecked(self.chosen.len() - 1).choice }
    }

    pub fn current_consequences(&self) -> &[(gen::LiteralSource, Literal)] {
        unsafe {
            &self
                .chosen
                .get_unchecked(self.chosen.len() - 1)
                .consequences
        }
    }

    pub fn forget_current_choice(&mut self) {
        self.chosen.pop();
    }

    pub fn proven_literals(&self) -> &[Literal] {
        &self.proven.observations
    }

    pub fn choice_made(&self) -> bool {
        !self.chosen.is_empty()
    }

    pub fn choice_count(&self) -> ChoiceIndex {
        self.chosen.len() as ChoiceIndex
    }
}

impl LiteralDB {
    fn top_mut(&mut self) -> &mut ChosenLiteral {
        let last_choice_index = self.chosen.len() - 1;
        unsafe { self.chosen.get_unchecked_mut(last_choice_index) }
    }
}
