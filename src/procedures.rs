use crate::structures::{Clause, Literal, VariableId};

use std::collections::BTreeSet;

/// Flattens an iterator over clauses to an iterator over the literals of some given polarity occuring in the clauses.
pub fn literals_of_polarity<'borrow>(
    clauses: impl Iterator<Item = &'borrow (impl Clause + 'borrow)> + 'borrow,
    polarity: bool,
) -> impl Iterator<Item = Literal> + 'borrow {
    clauses.flat_map(move |clause| {
        clause.literals().flat_map(move |literal| {
            if literal.polarity == polarity {
                Some(literal)
            } else {
                None
            }
        })
    })
}

/// General order for pairs related to booleans is 0 is false, 1 is true
pub fn hobson_choices<'borrow>(
    clauses: impl Iterator<Item = &'borrow (impl Clause + 'borrow)>,
) -> (Vec<VariableId>, Vec<VariableId>) {
    let mut the_true: BTreeSet<VariableId> = BTreeSet::new();
    let mut the_false: BTreeSet<VariableId> = BTreeSet::new();

    clauses.for_each(|clause| {
        clause.literals().for_each(|literal| {
            match literal.polarity {
                true => the_true.insert(literal.v_id),
                false => the_false.insert(literal.v_id),
            };
        })
    });

    let hobson_false: Vec<_> = the_false.difference(&the_true).cloned().collect();
    let hobson_true: Vec<_> = the_true.difference(&the_false).cloned().collect();
    (hobson_false, hobson_true)
}

pub fn binary_resolution<T: Clause>(cls_a: &T, cls_b: &T, v_id: VariableId) -> Option<impl Clause> {
    let mut the_clause = BTreeSet::new();
    let mut clause_a_value = None;
    let mut counterpart_found = false;
    for literal in cls_a.literals() {
        if literal.v_id == v_id
            && (clause_a_value.is_none() || clause_a_value == Some(literal.polarity))
        {
            clause_a_value = Some(literal.polarity);
        } else {
            the_clause.insert(literal);
        }
    }
    if clause_a_value.is_none() {
        log::warn!("Resolution: {v_id} not found in {}", cls_a.as_string());
        return None;
    }
    for literal in cls_b.literals() {
        if literal.v_id == v_id && clause_a_value != Some(literal.polarity) {
            counterpart_found = true;
        } else {
            the_clause.insert(literal);
        }
    }
    if !counterpart_found {
        log::warn!("Resolution: {v_id} not found in {}", cls_b.as_string());
        return None;
    }
    Some(the_clause.iter().cloned().collect::<Vec<_>>())
}
