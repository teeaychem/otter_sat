pub mod details;

use std::borrow::Borrow;

use crossbeam::channel::Sender;

use crate::{
    db::keys::ChoiceIndex,
    dispatch::{
        library::delta::{self},
        Dispatch,
    },
    structures::literal::Literal,
    types::gen::{self},
};

/*
A struct abstracting over decision levels.
Internally this makes use of a pair of private structs.
Though, this should probably be revised at some point…

- KnowledgeLevel
  Aka. decision level zero
  This contains assumptions or proven literals

- DecisionLevel
  A choice and the consequences of that choice

Specifically, each structs can be replaced by a simple vec.
And, for decision levels a stack of pointers to where the level began would work.
The choice/consequence distinction requires some attention, though.

For now, this works ok.
 */

pub struct LiteralDB {
    proven: ProvenLiterals,
    choice_stack: Vec<ChosenLiteral>,
    tx: Option<Sender<Dispatch>>,
}

#[derive(Debug)]
struct ProvenLiterals {
    observations: Vec<Literal>,
}

#[derive(Debug)]
struct ChosenLiteral {
    choice: Literal,
    consequences: Vec<(gen::src::Literal, Literal)>,
}

impl LiteralDB {
    pub fn new(tx: Option<Sender<Dispatch>>) -> Self {
        LiteralDB {
            proven: ProvenLiterals::default(),
            choice_stack: Vec::default(),
            tx,
        }
    }
}

impl LiteralDB {
    pub fn note_choice(&mut self, choice: Literal) {
        self.choice_stack.push(ChosenLiteral::new(choice));
    }

    /*
    A recorded literal may be the consequence of a choice or `proven`.
    In some cases this is simple to determine when the record happens.
    For example, if a literal was an (external) assumption it is `proven`.
    Still, in some cases it's easier to check when recording the literal.
    So, checks are made here.
    */
    pub fn record_literal(&mut self, literal: impl Borrow<Literal>, source: gen::src::Literal) {
        match source {
            gen::src::Literal::Choice => {}
            gen::src::Literal::Assumption => {
                if let Some(tx) = &self.tx {
                    let delta = delta::LiteralDB::Assumption(literal.borrow().to_owned());
                    tx.send(Dispatch::Delta(delta::Delta::LiteralDB(delta)));
                }
                self.proven.record_literal(literal)
            }
            gen::src::Literal::Pure => {
                if let Some(tx) = &self.tx {
                    let delta = delta::LiteralDB::Pure(literal.borrow().to_owned());
                    tx.send(Dispatch::Delta(delta::Delta::LiteralDB(delta)));
                }
                self.proven.record_literal(literal)
            }
            gen::src::Literal::BCP(_) => match self.choice_stack.len() {
                0 => {
                    if let Some(tx) = &self.tx {
                        let delta = delta::LiteralDB::Proof(literal.borrow().to_owned());
                        tx.send(Dispatch::Delta(delta::Delta::LiteralDB(delta)));
                    }
                    self.proven.record_literal(literal)
                }
                _ => self.top_mut().record_consequence(literal, source),
            },
            gen::src::Literal::Resolution(_) => {
                // Resoluion implies deduction via (known) clauses
                if let Some(tx) = &self.tx {
                    let delta = delta::LiteralDB::ResolutionProof(literal.borrow().to_owned());
                    tx.send(Dispatch::Delta(delta::Delta::LiteralDB(delta)));
                }
                self.proven.record_literal(literal)
            }
            gen::src::Literal::Forced(key) => match self.choice_stack.len() {
                0 => {
                    if let Some(tx) = &self.tx {
                        let delta = delta::LiteralDB::Forced(key, literal.borrow().to_owned());
                        tx.send(Dispatch::Delta(delta::Delta::LiteralDB(delta)));
                    }
                    self.proven.record_literal(literal)
                }
                _ => self.top_mut().record_consequence(literal, source),
            },
            gen::src::Literal::Missed(key) => match self.choice_stack.len() {
                0 => {
                    // TODO: Make unique o generalise forcing
                    if let Some(tx) = &self.tx {
                        let delta = delta::LiteralDB::Forced(key, literal.borrow().to_owned());
                        tx.send(Dispatch::Delta(delta::Delta::LiteralDB(delta)));
                    }
                    self.proven.record_literal(literal)
                }
                _ => self.top_mut().record_consequence(literal, source),
            },
        }
    }

    pub fn last_choice(&self) -> Literal {
        unsafe {
            self.choice_stack
                .get_unchecked(self.choice_stack.len() - 1)
                .choice
        }
    }

    pub fn last_consequences(&self) -> &[(gen::src::Literal, Literal)] {
        unsafe {
            &self
                .choice_stack
                .get_unchecked(self.choice_stack.len() - 1)
                .consequences
        }
    }

    pub fn forget_last_choice(&mut self) {
        self.choice_stack.pop();
    }

    pub fn proven_literals(&self) -> &[Literal] {
        &self.proven.observations
    }

    pub fn choice_made(&self) -> bool {
        !self.choice_stack.is_empty()
    }

    pub fn choice_count(&self) -> ChoiceIndex {
        self.choice_stack.len() as ChoiceIndex
    }
}
