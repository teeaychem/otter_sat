use crate::structures::{Clause, Literal, VariableId};

use std::collections::BTreeSet;

/// Flattens an iterator over clauses to an iterator over the literals of some given polarity occuring in the clauses.
/// Takes a count of possible variables as a vector of the literals is made at an intermediate stage to avoid some lifetime issues.
// TODO: avoid the intermediate vector
pub fn literals_of_polarity<'borrow>(
    clauses: impl Iterator<Item = &'borrow (impl Clause + 'borrow)>,
    var_count: usize,
    polarity: bool,
) -> impl Iterator<Item = Literal> {
    let mut literal_vec: Vec<Option<Literal>> = vec![None; var_count];
    clauses.for_each(|clause| {
        clause.literals().for_each(|literal| {
            if literal.polarity == polarity {
                literal_vec[literal.v_id] = Some(literal)
            }
        })
    });

    literal_vec.into_iter().flatten()
}

/// general order for pairs related to booleans is 0 is false, 1 is true
pub fn hobson_choices<'borrow>(
    clauses: impl Iterator<Item = &'borrow (impl Clause + 'borrow)>,
) -> (Vec<VariableId>, Vec<VariableId>) {
    let mut the_true: BTreeSet<VariableId> = BTreeSet::new();
    let mut the_false: BTreeSet<VariableId> = BTreeSet::new();

    clauses.for_each(|clause| {
        clause.literals().for_each(|literal| {
            match literal.polarity {
                true => the_true.insert(literal.v_id),
                false => the_false.insert(literal.v_id)
            };
        })
    });

    let hobson_false: Vec<_> = the_false.difference(&the_true).cloned().collect();
    let hobson_true: Vec<_> = the_true.difference(&the_false).cloned().collect();
    (hobson_false, hobson_true)

}
