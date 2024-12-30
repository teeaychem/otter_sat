pub mod details;

use std::rc::Rc;

use crate::{
    db::keys::ChoiceIndex,
    dispatch::Dispatch,
    structures::{
        atom::Atom,
        literal::{self, abLiteral, Literal},
    },
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
    pub choice_stack: Vec<ChosenLiteral>,
    pub dispatcher: Option<Rc<dyn Fn(Dispatch)>>,
}

#[derive(Debug)]
pub struct ChosenLiteral {
    choice: abLiteral,
    consequences: Vec<(literal::Source, abLiteral)>,
}

impl LiteralDB {
    pub fn new(tx: Option<Rc<dyn Fn(Dispatch)>>) -> Self {
        LiteralDB {
            choice_stack: Vec::default(),
            dispatcher: tx,
        }
    }
}

impl LiteralDB {
    pub fn note_choice(&mut self, choice: abLiteral) {
        self.choice_stack.push(ChosenLiteral::new(choice));
    }

    /*
    A recorded literal may be the consequence of a choice or `proven`.
    In some cases this is simple to determine when the record happens.
    For example, if a literal was an (external) assumption it is `proven`.
    Still, in some cases it's easier to check when recording the literal.
    So, checks are made here.
    */

    pub fn last_choice(&self) -> abLiteral {
        unsafe {
            self.choice_stack
                .get_unchecked(self.choice_stack.len() - 1)
                .choice
        }
    }

    pub fn last_consequences(&self) -> &[(literal::Source, abLiteral)] {
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

    pub fn choice_made(&self) -> bool {
        !self.choice_stack.is_empty()
    }

    pub fn choice_count(&self) -> ChoiceIndex {
        self.choice_stack.len() as ChoiceIndex
    }

    pub fn make_literal(&self, atoms: Atom, polarity: bool) -> abLiteral {
        abLiteral::fresh(atoms, polarity)
    }
}
