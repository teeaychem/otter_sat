use crate::structures::{
    clause::Clause,
    literal::Literal,
    valuation::{Valuation, ValuationWindow},
    variable::{Variable, VariableId},
};

pub type ClauseVec = Vec<Literal>;

impl Clause for ClauseVec {
    fn literals(&self) -> impl Iterator<Item = Literal> {
        self.iter().cloned()
    }

    fn variables(&self) -> impl Iterator<Item = VariableId> {
        self.iter().map(|literal| literal.v_id())
    }

    fn is_sat_on(&self, valuation: &ValuationWindow) -> bool {
        self.iter()
            .any(|l| valuation.of_v_id(l.v_id()) == Some(l.polarity()))
    }

    fn is_unsat_on(&self, valuation: &ValuationWindow) -> bool {
        self.iter().all(|l| {
            if let Some(var_valuie) = valuation.of_v_id(l.v_id()) {
                var_valuie != l.polarity()
            } else {
                false
            }
        })
    }

    fn find_unit_literal<T: Valuation>(&self, valuation: &T) -> Option<Literal> {
        let mut unit = None;

        for literal in self {
            let assigned_value = valuation.of_v_id(literal.v_id());
            if assigned_value.is_some_and(|v| v == literal.polarity()) {
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
            match valuation.of_v_id(literal.v_id()) {
                Some(value) if value == literal.polarity() => {
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

    fn as_dimacs(&self, variables: &[Variable]) -> String {
        let mut the_string = String::from("");
        for literal in self {
            let the_represenetation = match literal.polarity() {
                true => format!("{} ", variables[literal.index()].name()),
                false => format!("-{} ", variables[literal.index()].name()),
            };
            the_string.push_str(the_represenetation.as_str())
        }
        the_string += "0";
        the_string
    }

    fn to_vec(self) -> ClauseVec {
        self
    }

    fn length(&self) -> usize {
        self.len()
    }

    /// Returns the literal asserted by the clause on the given valuation
    fn asserts(&self, val: &impl Valuation) -> Option<Literal> {
        let mut the_literal = None;
        for lit in self.literals() {
            if let Some(existing_val) = val.of_v_id(lit.v_id()) {
                match existing_val == lit.polarity() {
                    true => return None,
                    false => continue,
                }
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
            .map(|l| vars[l.index()].decision_level())
            .collect::<Vec<_>>();
        decision_levels.sort_unstable();
        decision_levels.dedup();
        decision_levels.len()
    }

    fn is_empty(&self) -> bool {
        self.is_empty()
    }

    /// Returns Some(literal) whose variable id matches the given id
    /// Uses binary search on longer clauses, as literals are ordered by variable ids
    fn find_literal_by_id(&self, id: VariableId) -> Option<Literal> {
        if self.len() < 64 {
            self.iter().find(|l| l.v_id() == id).copied()
        } else {
            find_literal_by_id_binary(self, id)
        }
    }
}

fn find_literal_by_id_binary(clause: &[Literal], id: VariableId) -> Option<Literal> {
    let mut min = 0;
    let mut max = clause.len() - 1;
    let mut midpoint;
    let mut attempt;
    loop {
        midpoint = min + ((max - min) / 2);
        attempt = clause[midpoint];
        if max - min == 0 {
            match attempt.v_id() == id {
                true => return Some(attempt),
                false => return None,
            }
        }
        match attempt.v_id().cmp(&id) {
            std::cmp::Ordering::Less => min = midpoint + 1,
            std::cmp::Ordering::Equal => {
                return Some(attempt);
            }
            std::cmp::Ordering::Greater => max = midpoint - 1,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::procedures::resolve_sorted_clauses;

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
        let resolution = resolve_sorted_clauses(a.literals(), b.literals(), 1);
        if let Some(resolved) = resolution {
            assert_eq!(
                vec![
                    Literal::new(2, false),
                    Literal::new(3, true),
                    Literal::new(4, false)
                ],
                resolved.to_vec()
            )
        } else {
            panic!("No resolution")
        }
    }

    #[test]
    fn resolve_nok_check() {
        let a = vec![Literal::new(1, true), Literal::new(2, false)];
        let b = vec![Literal::new(3, true), Literal::new(4, false)];
        assert!(resolve_sorted_clauses(a.literals(), b.literals(), 1).is_none())
    }

    #[test]
    fn find_check_one() {
        let test_clause = vec![Literal::new(1, true)];
        assert_eq!(
            Some(Literal::new(1, true)),
            test_clause.find_literal_by_id(1)
        );
        assert_eq!(None, test_clause.find_literal_by_id(2));
    }

    #[test]
    fn find_check_multiple_found() {
        let test_clause = vec![
            Literal::new(1, true),
            Literal::new(2, false),
            Literal::new(3, false),
            Literal::new(4, false),
            Literal::new(7, false),
        ];
        assert_eq!(
            Some(Literal::new(1, true)),
            find_literal_by_id_binary(&test_clause, 1)
        );
        assert_eq!(
            Some(Literal::new(2, false)),
            find_literal_by_id_binary(&test_clause, 2)
        );
        assert_eq!(
            Some(Literal::new(4, false)),
            test_clause.find_literal_by_id(4)
        );
        assert_eq!(None, find_literal_by_id_binary(&test_clause, 5));
        assert_eq!(None, find_literal_by_id_binary(&test_clause, 6));
        assert_eq!(
            Some(Literal::new(7, false)),
            find_literal_by_id_binary(&test_clause, 7)
        );
    }
}
