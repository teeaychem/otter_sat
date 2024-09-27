use crate::structures::{Literal, Valuation, ValuationVec, Variable, VariableId};

pub type ClauseVec = Vec<Literal>;

pub trait Clause: IntoIterator {
    fn literals(&self) -> impl Iterator<Item = Literal>;

    fn variables(&self) -> impl Iterator<Item = VariableId>;

    fn is_sat_on(&self, valuation: &ValuationVec) -> bool;

    fn is_unsat_on(&self, valuation: &ValuationVec) -> bool;

    fn find_unit_literal<T: Valuation>(&self, valuation: &T) -> Option<Literal>;

    fn collect_choices<T: Valuation>(&self, valuation: &T) -> Option<Vec<Literal>>;

    fn as_string(&self) -> String;

    fn is_empty(&self) -> bool;

    fn as_vec(&self) -> ClauseVec;

    fn to_vec(self) -> ClauseVec;

    fn to_sorted_vec(self) -> ClauseVec;

    fn len(&self) -> usize;

    fn asserts(&self, val: &impl Valuation) -> Option<Literal>;

    fn lbd(&self, variables: &[Variable]) -> usize;
}

pub type ClauseId = usize;

impl Clause for ClauseVec {
    fn literals(&self) -> impl Iterator<Item = Literal> {
        self.iter().cloned()
    }

    fn variables(&self) -> impl Iterator<Item = VariableId> {
        self.iter().map(|literal| literal.v_id)
    }

    fn is_sat_on(&self, valuation: &ValuationVec) -> bool {
        self.iter()
            .any(|l| valuation.of_v_id(l.v_id) == Some(l.polarity))
    }

    fn is_unsat_on(&self, valuation: &ValuationVec) -> bool {
        self.iter().all(|l| {
            if let Some(var_valuie) = valuation.of_v_id(l.v_id) {
                var_valuie != l.polarity
            } else {
                false
            }
        })
    }

    fn find_unit_literal<T: Valuation>(&self, valuation: &T) -> Option<Literal> {
        let mut unit = None;

        for literal in self {
            let assigned_value = valuation.of_v_id(literal.v_id);
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
        unit
    }

    fn collect_choices<T: Valuation>(&self, valuation: &T) -> Option<Vec<Literal>> {
        let mut the_literals = vec![];

        for literal in self {
            match valuation.of_v_id(literal.v_id) {
                Some(value) if value == literal.polarity => {
                    return None;
                }
                Some(_value) => continue,

                None => the_literals.push(*literal),
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

    fn len(&self) -> usize {
        self.len()
    }

    /// Returns the literal asserted by the clause on the given valuation
    fn asserts(&self, val: &impl Valuation) -> Option<Literal> {
        let mut the_literal = None;
        for lit in self.literals() {
            if val.of_v_id(lit.v_id).is_some_and(|p| p == lit.polarity) {
                return None;
            } else if the_literal.is_none() {
                the_literal = Some(lit);
            } else {
                return None;
            }
        }
        the_literal
    }

    // TODO: consider a different approach to lbd
    // e.g. an approximate measure of =2, =3, >4 can be settled much more easily
    fn lbd(&self, vars: &[Variable]) -> usize {
        let mut decision_levels = self
            .iter()
            .map(|l| vars[l.v_id].decision_level())
            .collect::<Vec<_>>();
        decision_levels.sort_unstable();
        decision_levels.dedup();
        decision_levels.len()
    }

    fn is_empty(&self) -> bool {
        self.is_empty()
    }
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
        let resolution = resolve_sorted_clauses(&a, &b, 1);
        if let Some(resolved) = resolution {
            assert_eq!(
                vec![
                    Literal::new(2, false),
                    Literal::new(3, true),
                    Literal::new(4, false)
                ],
                resolved.to_sorted_vec()
            )
        } else {
            panic!("No resolution")
        }
    }

    #[test]
    fn resolve_nok_check() {
        let a = vec![Literal::new(1, true), Literal::new(2, false)];
        let b = vec![Literal::new(3, true), Literal::new(4, false)];
        assert!(resolve_sorted_clauses(&a, &b, 1).is_none())
    }
}
