use crate::structures::{clause::Clause, literal::Literal, variable::VariableId};

use std::collections::BTreeSet;

/// General order for pairs related to booleans is 0 is false, 1 is true
pub fn hobson_choices<'borrow>(
    clauses: impl Iterator<Item = impl Iterator <Item = Literal>>,
) -> (Vec<VariableId>, Vec<VariableId>) {
    let mut the_true: BTreeSet<VariableId> = BTreeSet::new();
    let mut the_false: BTreeSet<VariableId> = BTreeSet::new();

    clauses.for_each(|literals| {
        literals.for_each(|literal| {
            match literal.polarity() {
                true => the_true.insert(literal.v_id()),
                false => the_false.insert(literal.v_id()),
            };
        })
    });

    let hobson_false: Vec<_> = the_false.difference(&the_true).cloned().collect();
    let hobson_true: Vec<_> = the_true.difference(&the_false).cloned().collect();
    (hobson_false, hobson_true)
}

pub fn resolve_sorted_clauses(
    mut clause_a_literals: impl Iterator<Item = Literal>,
    mut clause_b_literals: impl Iterator<Item = Literal>,
    v_id: VariableId,
) -> Option<impl Clause> {
    let mut current_a = clause_a_literals.next();
    let mut current_b = clause_b_literals.next();

    let mut the_clause = vec![];
    let mut a_found: Option<bool> = None;
    let mut b_found: Option<bool> = None;

    loop {
        match (current_a, current_b) {
            (None, None) => break,
            (Some(a_lit), None) => {
                if a_lit.v_id() == v_id {
                    if let Some(existing_b) = b_found {
                        if existing_b != a_lit.polarity() {
                            a_found = Some(a_lit.polarity());
                        } else {
                            return None;
                        }
                    } else {
                        a_found = Some(a_lit.polarity());
                    }
                } else {
                    the_clause.push(a_lit);
                }
                current_a = clause_a_literals.next();
            }
            (None, Some(b_lit)) => {
                if b_lit.v_id() == v_id {
                    if let Some(existing) = a_found {
                        if existing != b_lit.polarity() {
                            b_found = Some(b_lit.polarity());
                        } else {
                            return None;
                        }
                    } else {
                        b_found = Some(b_lit.polarity());
                    }
                } else {
                    the_clause.push(b_lit);
                }
                current_b = clause_b_literals.next();
            }
            (Some(a_lit), Some(b_lit)) => {
                if a_lit.v_id() == v_id {
                    if let Some(existing) = b_found {
                        if existing != a_lit.polarity() {
                            a_found = Some(a_lit.polarity());
                        } else {
                            return None;
                        }
                    } else {
                        a_found = Some(a_lit.polarity());
                    }
                    current_a = clause_a_literals.next();
                } else if b_lit.v_id() == v_id {
                    if let Some(existing) = a_found {
                        if existing != b_lit.polarity() {
                            b_found = Some(b_lit.polarity());
                        } else {
                            return None;
                        }
                    } else {
                        b_found = Some(b_lit.polarity());
                    }
                    current_b = clause_b_literals.next();
                } else {
                    match a_lit.cmp(&b_lit) {
                        std::cmp::Ordering::Equal => {
                            the_clause.push(a_lit);
                            current_a = clause_a_literals.next();
                            current_b = clause_b_literals.next();
                        }
                        std::cmp::Ordering::Less => {
                            the_clause.push(a_lit);
                            current_a = clause_a_literals.next();
                        }
                        std::cmp::Ordering::Greater => {
                            the_clause.push(b_lit);
                            current_b = clause_b_literals.next();
                        }
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

/// Work through two ordered vectors, noting any occurrences of the same variable under contrastring polarity
pub fn find_counterpart_literals(
    mut cls_a: impl Iterator<Item = Literal>,
    mut cls_b: impl Iterator<Item = Literal>,
) -> Vec<VariableId> {
    let mut candidates = vec![];

    let mut current_a = cls_a.next();
    let mut current_b = cls_b.next();

    while let (Some(a_lit), Some(b_lit)) = (current_a, current_b) {
        if a_lit.v_id() == b_lit.v_id() {
            if a_lit.polarity() != b_lit.polarity() {
                candidates.push(a_lit.v_id());
            }
            current_a = cls_a.next();
            current_b = cls_b.next();
        } else if a_lit < b_lit {
            current_a = cls_a.next();
        } else if b_lit < a_lit {
            current_b = cls_b.next();
        } else {
            panic!("Incomparable literals found");
        }
    }

    candidates
}

#[cfg(test)]
mod tests {
    use super::*;

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
        let result = resolve_sorted_clauses(a.literals(), b.literals(), 1);
        assert!(result.is_some());

        assert_eq!(
            vec![
                Literal::new(2, false),
                Literal::new(3, true),
                Literal::new(4, false),
                Literal::new(4, true)
            ],
            result.unwrap().to_vec()
        )
    }

    #[test]
    fn sorted_resolve_nok_check() {
        let a = vec![Literal::new(1, true), Literal::new(2, false)];
        let b = vec![Literal::new(3, true), Literal::new(4, false)];
        assert!(resolve_sorted_clauses(a.literals(), b.literals(), 1).is_none())
    }
}
