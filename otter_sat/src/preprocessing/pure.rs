//! Procuedures to identify pure literals.
use std::collections::HashSet;

use crate::{
    context::GenericContext,
    db::consequence_q::{self},
    structures::{
        atom::Atom,
        clause::Clause,
        consequence::{self, Consequence},
        literal::{CLiteral, Literal},
    },
    types::err::{self},
};

// General order for pairs related to booleans is 0 is false, 1 is true
/// Given an interator over clauses returns a pair of iterators over the pure literals relative to those clauses.
///
/// In other words, returns a pair of iterators where the first iterator contains all the literals which occur only with positive polarity and the second iterator contains all the literals which occur only with negative polarity.
pub fn pure_literals<'l>(
    clauses: impl Iterator<Item = impl Iterator<Item = CLiteral>>,
) -> (Vec<Atom>, Vec<Atom>) {
    let mut the_true: HashSet<Atom> = HashSet::new();
    let mut the_false: HashSet<Atom> = HashSet::new();

    clauses.for_each(|literals| {
        for literal in literals {
            match literal.polarity() {
                true => the_true.insert(literal.atom()),
                false => the_false.insert(literal.atom()),
            };
        }
    });

    let pure_false: Vec<_> = the_false.difference(&the_true).copied().collect();
    let pure_true: Vec<_> = the_true.difference(&the_false).copied().collect();
    (pure_false, pure_true)
}

/// Finds all pure literals with respect to non-unit clauses and sets the value of the relevant atom to match the polarity of the literal.
pub fn set_pure<R: rand::Rng + std::default::Default>(
    context: &mut GenericContext<R>,
) -> Result<(), err::ConsequenceQueueError> {
    let (f, t) = pure_literals(
        context
            .clause_db
            .all_nonunit_clauses()
            .map(|(_, sc)| sc.literals()),
    );

    for atom in f.into_iter() {
        let the_literal = CLiteral::new(atom, false);
        let position = consequence_q::QPosition::Back;
        let level = context.literal_db.decision_level();

        match context.value_and_queue(the_literal, position, level) {
            Ok(consequence_q::ConsequenceQueueOk::Qd) => {
                let consequence = Consequence::from(the_literal, consequence::Source::PureLiteral);
                context.record_consequence(consequence);
            }
            Ok(consequence_q::ConsequenceQueueOk::Skip) => {}

            Err(e) => return Err(e),
        }
    }

    for atom in t.into_iter() {
        let the_literal = CLiteral::new(atom, false);
        let position = consequence_q::QPosition::Back;
        let level = context.literal_db.decision_level();

        match context.value_and_queue(the_literal, position, level) {
            Ok(consequence_q::ConsequenceQueueOk::Qd) => {
                let consequence = Consequence::from(the_literal, consequence::Source::PureLiteral);
                context.record_consequence(consequence);
            }
            Ok(consequence_q::ConsequenceQueueOk::Skip) => {}

            Err(e) => return Err(e),
        }
    }

    Ok(())
}
