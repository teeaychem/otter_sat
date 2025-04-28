/*!
Procuedures to identify literals which occur only positive or only negatively in a formula --- a.k.a 'pure' literals.


*/
use std::collections::HashSet;

use crate::{
    context::GenericContext,
    structures::{
        atom::Atom,
        clause::Clause,
        consequence::AssignmentSource,
        literal::{CLiteral, Literal},
        valuation::ValuationStatus,
    },
    types::err::{self, PreprocessingError},
};

// General order for pairs related to booleans is 0 is false, 1 is true
/// Given an iterator over clauses returns a pair of iterators over the pure literals relative to those clauses.
///
/// In other words, returns a pair of iterators where the first iterator contains all the literals which occur only with positive polarity and the second iterator contains all the literals which occur only with negative polarity.
pub fn pure_literals(
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

/// Finds all pure literals in non-unit clauses and assigns the polarity to the atom.
pub fn set_pure<R: rand::Rng + std::default::Default>(
    context: &mut GenericContext<R>,
) -> Result<(), PreprocessingError> {
    let (f, t) = pure_literals(
        context
            .clause_db
            .all_nonunit_clauses()
            .map(|(_, sc)| sc.literals()),
    );

    for literal in f
        .into_iter()
        .map(|atom| CLiteral::new(atom, false))
        .chain(t.into_iter().map(|atom| CLiteral::new(atom, false)))
    {
        match context.check_assignment(literal) {
            ValuationStatus::None => {
                context.record_assignment(literal, AssignmentSource::Pure);
            }

            ValuationStatus::Set => {}

            ValuationStatus::Conflict => return Err(err::PreprocessingError::Unsatisfiable),
        }
    }

    Ok(())
}
