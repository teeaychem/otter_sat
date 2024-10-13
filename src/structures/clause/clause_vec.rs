use crate::structures::{
    clause::Clause, literal::Literal, valuation::Valuation, variable::Variable,
};

pub type ClauseVec = Vec<Literal>;

impl Clause for ClauseVec {
    fn literals(&self) -> impl Iterator<Item = Literal> {
        self.iter().copied()
    }

    fn as_string(&self) -> String {
        let mut the_string = String::from("(");
        for literal in self {
            the_string.push_str(format!(" {literal} ").as_str());
        }
        the_string += ")";
        the_string
    }

    fn as_dimacs(&self, variables: &[Variable]) -> String {
        let mut the_string = String::new();
        for literal in self {
            let the_represenetation = match literal.polarity() {
                true => format!("{} ", variables[literal.index()].name()),
                false => format!("-{} ", variables[literal.index()].name()),
            };
            the_string.push_str(the_represenetation.as_str());
        }
        the_string += "0";
        the_string
    }

    fn to_clause_vec(self) -> ClauseVec {
        self
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
        match resolution {
            Some(resolved) => {
                assert_eq!(
                    vec![
                        Literal::new(2, false),
                        Literal::new(3, true),
                        Literal::new(4, false)
                    ],
                    resolved.to_clause_vec()
                )
            }
            None => panic!("No resolution"),
        }
    }

    #[test]
    fn resolve_nok_check() {
        let a = vec![Literal::new(1, true), Literal::new(2, false)];
        let b = vec![Literal::new(3, true), Literal::new(4, false)];
        assert!(resolve_sorted_clauses(a.literals(), b.literals(), 1).is_none())
    }
}
