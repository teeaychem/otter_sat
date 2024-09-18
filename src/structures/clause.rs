use crate::structures::{Literal, LiteralError, Valuation, ValuationVec, VariableId};

use std::collections::BTreeSet;

pub type ClauseVec = Vec<Literal>;

pub trait Clause: IntoIterator {
    fn add_literal(&mut self, literal: Literal);

    fn literals(&self) -> impl Iterator<Item = Literal>;

    fn variables(&self) -> impl Iterator<Item = VariableId>;

    fn is_sat_on(&self, valuation: &ValuationVec) -> bool;

    fn is_unsat_on(&self, valuation: &ValuationVec) -> bool;

    fn find_unit_literal<T: Valuation>(&self, valuation: &T) -> Option<Literal>;

    fn collect_choices<T: Valuation>(&self, valuation: &T) -> Option<Vec<Literal>>;

    fn as_string(&self) -> String;

    fn as_vec(&self) -> ClauseVec;

    fn to_vec(self) -> ClauseVec;

    fn to_sorted_vec(self) -> ClauseVec;
}

pub type ClauseId = usize;

impl Clause for ClauseVec {
    fn add_literal(&mut self, literal: Literal) {
        self.push(literal);
    }

    fn literals(&self) -> impl Iterator<Item = Literal> {
        self.iter().cloned()
    }

    fn variables(&self) -> impl Iterator<Item = VariableId> {
        self.iter().map(|literal| literal.v_id)
    }

    fn is_sat_on(&self, valuation: &ValuationVec) -> bool {
        self.iter()
            .any(|l| valuation.of_v_id(l.v_id) == Ok(Some(l.polarity)))
    }

    fn is_unsat_on(&self, valuation: &ValuationVec) -> bool {
        self.iter().all(|l| {
            if let Ok(Some(var_valuie)) = valuation.of_v_id(l.v_id) {
                var_valuie != l.polarity
            } else {
                false
            }
        })
    }

    fn find_unit_literal<T: Valuation>(&self, valuation: &T) -> Option<Literal> {
        let mut unit = None;

        for literal in self {
            if let Ok(assigned_value) = valuation.of_v_id(literal.v_id) {
                if assigned_value.is_some_and(|v| v == literal.polarity) {
                    // the clause is satisfied and so does not provide any new information
                    break;
                } else if assigned_value.is_some() {
                    // either every literal so far has been valued the opposite, or there has been exactly on unvalued literal, so continue
                    continue;
                } else {
                    // if no other literal has been found then this literal may be unit, so mark it and continue
                    // though, if some other literal has already been marked, the clause does not force any literal
                    match unit {
                        Some(_) => {
                            unit = None;
                            break;
                        }
                        None => unit = Some(*literal),
                    }
                }
            }
        }
        unit
    }

    fn collect_choices<T: Valuation>(&self, valuation: &T) -> Option<Vec<Literal>> {
        let mut the_literals = vec![];

        for literal in self {
            match valuation.of_v_id(literal.v_id) {
                Ok(assigned_value) => match assigned_value {
                    Some(value) if value == literal.polarity => {
                        return None;
                    }
                    Some(_value) => continue,

                    None => the_literals.push(*literal),
                },
                Err(_) => panic!("Failed to get valuation of variable"),
            }
        }
        Some(the_literals)
    }

    fn as_string(&self) -> String {
        let mut the_string = String::from("(");
        for literal in self {
            the_string.push_str(format!(" {} ", literal).as_str())
        }
        the_string += ")";
        the_string
    }

    fn as_vec(&self) -> ClauseVec {
        self.clone()
    }

    fn to_vec(self) -> ClauseVec {
        self
    }

    fn to_sorted_vec(mut self) -> ClauseVec {
        self.sort();
        self
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolve_ok_check() {
        let a = vec![
            Literal::new(1, true),
            Literal::new(2, false),
            Literal::new(4, false),
        ];
        let b = vec![
            Literal::new(1, false),
            Literal::new(3, true),
            Literal::new(4, false),
        ];
        assert_eq!(
            vec![
                Literal::new(2, false),
                Literal::new(3, true),
                Literal::new(4, false)
            ],
            binary_resolution(&a, &b, 1).unwrap().to_sorted_vec()
        )
    }

    #[test]
    fn resolve_nok_check() {
        let a = vec![Literal::new(1, true), Literal::new(2, false)];
        let b = vec![Literal::new(3, true), Literal::new(4, false)];
        assert!(binary_resolution(&a, &b, 1).is_none())
    }
}
