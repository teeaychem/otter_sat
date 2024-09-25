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
        if literal.v_id != v_id {
            the_clause.insert(literal);
        } else if clause_a_value.is_none() {
            clause_a_value = Some(literal.polarity);
        }
    }
    if clause_a_value.is_none() {
        log::error!("Resolution: {v_id} not found in {}", cls_a.as_string());
        return None;
    }
    for literal in cls_b.literals() {
        if literal.v_id != v_id {
            the_clause.insert(literal);
        } else if !counterpart_found && Some(literal.polarity) != clause_a_value {
            counterpart_found = true;
        }
    }
    if !counterpart_found {
        log::error!("Resolution: {v_id} not found in {}", cls_b.as_string());
        return None;
    }
    Some(the_clause.iter().cloned().collect::<Vec<_>>())
}

pub fn merge_sorted_clauses<T: Clause>(cls_a: &T, cls_b: &T) -> impl Clause {
    let mut the_clause = vec![];
    let mut a_ptr = 0;
    let mut b_ptr = 0;
    let a_vec = cls_a.as_vec();
    let a_max = a_vec.len();
    let b_vec = cls_b.as_vec();
    let b_max = b_vec.len();

    loop {
        if a_ptr >= a_max && b_ptr >= b_max {
            break;
        } else if a_ptr < a_max && b_ptr >= b_max {
            the_clause.push(a_vec[a_ptr]);
            a_ptr += 1;
        } else if a_ptr >= a_max && b_ptr < b_max {
            the_clause.push(b_vec[b_ptr]);
            b_ptr += 1;
        } else if a_ptr < a_max && b_ptr < b_max {
            let a_lit = a_vec[a_ptr];
            let b_lit = b_vec[b_ptr];

            match a_lit.cmp(&b_lit) {
                std::cmp::Ordering::Equal => {
                    the_clause.push(a_lit);
                    a_ptr += 1;
                    b_ptr += 1;
                }
                std::cmp::Ordering::Less => {
                    the_clause.push(a_lit);
                    a_ptr += 1;
                }
                std::cmp::Ordering::Greater => {
                    the_clause.push(b_lit);
                    b_ptr += 1;
                }
            }
        }
    }
    the_clause.dedup();

    the_clause
}

pub fn resolve_sorted_clauses<T: Clause>(
    cls_a: &T,
    cls_b: &T,
    v_id: VariableId,
) -> Option<impl Clause> {
    let mut the_clause = vec![];
    let mut a_ptr = 0;
    let mut b_ptr = 0;
    let a_vec = cls_a.as_vec();
    let a_max = a_vec.len();
    let b_vec = cls_b.as_vec();
    let b_max = b_vec.len();
    let mut a_found = None;
    let mut b_found = None;

    loop {
        if a_ptr >= a_max && b_ptr >= b_max {
            break;
        } else if a_ptr < a_max && b_ptr >= b_max {
            let a_lit = a_vec[a_ptr];
            if a_lit.v_id == v_id {
                if let Some(existing) = b_found {
                    if existing != a_lit.polarity {
                        a_found = Some(a_lit.polarity);
                        a_ptr += 1;
                    } else {
                        return None;
                    }
                } else {
                    a_found = Some(a_lit.polarity);
                    a_ptr += 1;
                }
            } else {
                the_clause.push(a_lit);
                a_ptr += 1;
            }
        } else if a_ptr >= a_max && b_ptr < b_max {
            let b_lit = b_vec[b_ptr];
            if b_lit.v_id == v_id {
                if let Some(existing) = a_found {
                    if existing != b_lit.polarity {
                        b_found = Some(b_lit.polarity);
                        b_ptr += 1;
                    } else {
                        return None;
                    }
                } else {
                    b_found = Some(b_lit.polarity);
                    b_ptr += 1;
                }
            } else {
                the_clause.push(b_lit);
                b_ptr += 1;
            }
        } else if a_ptr < a_max && b_ptr < b_max {
            let a_lit = a_vec[a_ptr];
            let b_lit = b_vec[b_ptr];
            if a_lit.v_id == v_id {
                if let Some(existing) = b_found {
                    if existing != a_lit.polarity {
                        a_found = Some(a_lit.polarity);
                        a_ptr += 1;
                    } else {
                        return None;
                    }
                } else {
                    a_found = Some(a_lit.polarity);
                    a_ptr += 1;
                }
            } else if b_lit.v_id == v_id {
                if let Some(existing) = a_found {
                    if existing != b_lit.polarity {
                        b_found = Some(b_lit.polarity);
                        b_ptr += 1;
                    } else {
                        return None;
                    }
                } else {
                    b_found = Some(b_lit.polarity);
                    b_ptr += 1;
                }
            } else {
                match a_lit.cmp(&b_lit) {
                    std::cmp::Ordering::Equal => {
                        the_clause.push(a_lit);
                        a_ptr += 1;
                        b_ptr += 1;
                    }
                    std::cmp::Ordering::Less => {
                        the_clause.push(a_lit);
                        a_ptr += 1;
                    }
                    std::cmp::Ordering::Greater => {
                        the_clause.push(b_lit);
                        b_ptr += 1;
                    }
                }
            }
        }
    }
    the_clause.dedup();

    if a_found.is_none() || b_found.is_none() {
        None
    } else {
        Some(the_clause)
    }
}

/// Initial placeholder implementation
pub fn find_counterpart_literals_placeholder<T: Clause>(cls_a: &T, cls_b: &T) -> Vec<VariableId> {
    let mut candidates = vec![];
    let var_a = cls_a.variables();
    let var_b = cls_b.variables();
    let a_set = var_a.collect::<BTreeSet<_>>();
    let b_set = var_b.collect::<BTreeSet<_>>();
    let both = a_set.intersection(&b_set);
    for v_id in both {
        let cls_a_lit = cls_a.literals().find(|l| l.v_id == *v_id).unwrap();
        let cls_b_lit = cls_b.literals().find(|l| l.v_id == *v_id).unwrap();
        if cls_a_lit.polarity != cls_b_lit.polarity {
            candidates.push(*v_id)
        }
    }
    candidates
}

/// Work through two ordered vectors, noting any occurrences of the same variable under contrastring polarity
pub fn find_counterpart_literals<T: Clause>(cls_a: &T, cls_b: &T) -> Vec<VariableId> {
    let mut candidates = vec![];
    let a_vec = cls_a.as_vec();
    let mut a_ptr = 0;
    let a_max = cls_a.len();
    let mut b_ptr = 0;
    let b_vec = cls_b.as_vec();
    let b_max = cls_b.len();
    loop {
        if a_ptr >= a_max || b_ptr >= b_max {
            break;
        }
        let a_lit = a_vec[a_ptr];
        let b_lit = b_vec[b_ptr];
        if a_lit.v_id == b_lit.v_id {
            if a_lit.polarity != b_lit.polarity {
                candidates.push(a_lit.v_id);
            }
            a_ptr += 1;
            b_ptr += 1;
        } else if a_lit < b_lit {
            a_ptr += 1;
        } else if b_lit < a_lit {
            b_ptr += 1;
        }
    }

    candidates
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn merge_check_one() {
        let a = vec![
            Literal::new(1, true),
            Literal::new(2, false),
            Literal::new(4, true),
        ];
        let b = vec![
            Literal::new(1, false),
            Literal::new(3, true),
            Literal::new(4, false),
        ];
        assert_eq!(
            vec![
                Literal::new(1, false),
                Literal::new(1, true),
                Literal::new(2, false),
                Literal::new(3, true),
                Literal::new(4, false),
                Literal::new(4, true)
            ],
            merge_sorted_clauses(&a, &b).as_vec()
        )
    }

    #[test]
    fn merge_check_two() {
        let a = vec![Literal::new(1, true), Literal::new(1, true)];
        let b = vec![Literal::new(2, false), Literal::new(2, true)];
        assert_eq!(
            vec![
                Literal::new(1, true),
                Literal::new(2, false),
                Literal::new(2, true),
            ],
            merge_sorted_clauses(&a, &b).as_vec()
        )
    }

    #[test]
    fn sorted_resolve_ok_check() {
        let a = vec![
            Literal::new(1, true),
            Literal::new(2, false),
            Literal::new(4, true),
        ];
        let b = vec![
            Literal::new(1, false),
            Literal::new(3, true),
            Literal::new(4, false),
        ];
        let result = resolve_sorted_clauses(&a, &b, 1);
        assert!(result.is_some());

        assert_eq!(
            vec![
                Literal::new(2, false),
                Literal::new(3, true),
                Literal::new(4, false),
                Literal::new(4, true)
            ],
            result.unwrap().as_vec()
        )
    }

    #[test]
    fn sorted_resolve_nok_check() {
        let a = vec![Literal::new(1, true), Literal::new(2, false)];
        let b = vec![Literal::new(3, true), Literal::new(4, false)];
        assert!(resolve_sorted_clauses(&a, &b, 1).is_none())
    }
}
