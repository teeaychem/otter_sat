//! Procuedures to identify pure literals.
use std::collections::BTreeSet;

use crate::{
    context::GenericContext,
    db::consequence_q::{self},
    structures::{
        atom::Atom,
        clause::Clause,
        literal::{self, abLiteral, Literal},
    },
    types::err::{self},
};

// General order for pairs related to booleans is 0 is false, 1 is true
/// Given an interator over clauses returns a pair of iterators over the pure literals relative to those clauses.
///
/// In other words, returns a pair of iterators where the first iterator contains all the literals which occur only with positive polarity and the second iterator contains all the literals which occur only with negative polarity.
pub fn pure_literals<'l>(
    clauses: impl Iterator<Item = impl Iterator<Item = &'l abLiteral>>,
) -> (Vec<Atom>, Vec<Atom>) {
    let mut the_true: BTreeSet<Atom> = BTreeSet::new();
    let mut the_false: BTreeSet<Atom> = BTreeSet::new();

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
) -> Result<(), err::Queue> {
    let (f, t) = pure_literals(
        context
            .clause_db
            .all_nonunit_clauses()
            .map(|sc| sc.literals()),
    );

    for atom in f.into_iter() {
        let the_literal = abLiteral::fresh(atom, false);
        match context.q_literal(
            the_literal,
            consequence_q::QPosition::Back,
            context.literal_db.decision_count(),
        ) {
            Ok(consequence_q::Ok::Qd) => {
                context.record_literal(the_literal, literal::Source::PureLiteral);
            }
            Ok(consequence_q::Ok::Skip) => {}

            Err(e) => return Err(e),
        }
    }

    for atom in t.into_iter() {
        let the_literal = abLiteral::fresh(atom, true);
        match context.q_literal(
            the_literal,
            consequence_q::QPosition::Back,
            context.literal_db.decision_count(),
        ) {
            Ok(consequence_q::Ok::Qd) => {
                context.record_literal(the_literal, literal::Source::PureLiteral)
            }

            Ok(consequence_q::Ok::Skip) => {}
            Err(e) => return Err(e),
        }
    }
    Ok(())
}
