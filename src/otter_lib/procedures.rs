use crate::structures::{literal::Literal, variable::VariableId};

use std::collections::BTreeSet;

/// General order for pairs related to booleans is 0 is false, 1 is true
pub fn hobson_choices(
    clauses: impl Iterator<Item = impl Iterator<Item = Literal>>,
) -> (Vec<VariableId>, Vec<VariableId>) {
    let mut the_true: BTreeSet<VariableId> = BTreeSet::new();
    let mut the_false: BTreeSet<VariableId> = BTreeSet::new();

    clauses.for_each(|literals| {
        literals.for_each(|literal| {
            match literal.polarity() {
                true => the_true.insert(literal.v_id()),
                false => the_false.insert(literal.v_id()),
            };
        });
    });

    let hobson_false: Vec<_> = the_false.difference(&the_true).copied().collect();
    let hobson_true: Vec<_> = the_true.difference(&the_false).copied().collect();
    (hobson_false, hobson_true)
}

// with help from https://github.com/aimacode/aima-python/blob/master/improving_sat_algorithms.ipynb
pub fn luby(i: usize) -> usize {
    let mut k = 1;
    loop {
        if i == (1_usize.wrapping_shl(k)) - 1 {
            return 1_usize.wrapping_shl(k - 1);
        } else if (1_usize.wrapping_shl(k - 1)) <= i && i < (1_usize.wrapping_shl(k)) - 1 {
            return luby(i - (1 << (k - 1)) + 1);
        } else {
            k += 1;
        }
    }
}
