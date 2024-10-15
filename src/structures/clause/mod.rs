pub mod stored;

use crate::structures::{literal::Literal, valuation::Valuation, variable::Variable};
use std::ops::Deref;

pub trait Clause {
    fn as_string(&self) -> String;

    fn as_dimacs(&self, variables: &[Variable]) -> String;

    fn asserts(&self, val: &impl Valuation) -> Option<Literal>;

    fn lbd(&self, variables: &[Variable]) -> usize;

    fn literal_slice(&self) -> &[Literal];

    fn length(&self) -> usize;
}

impl<T: Deref<Target = [Literal]>> Clause for T {
    fn literal_slice(&self) -> &[Literal] {
        self
    }

    fn as_string(&self) -> String {
        let mut the_string = String::from("(");
        for literal in self.literal_slice() {
            the_string.push_str(format!(" {literal} ").as_str());
        }
        the_string += ")";
        the_string
    }

    fn as_dimacs(&self, variables: &[Variable]) -> String {
        let mut the_string = String::new();
        for literal in self.literal_slice() {
            let the_represenetation = match literal.polarity() {
                true => format!("{} ", variables[literal.index()].name()),
                false => format!("-{} ", variables[literal.index()].name()),
            };
            the_string.push_str(the_represenetation.as_str());
        }
        the_string += "0";
        the_string
    }

    /// Returns the literal asserted by the clause on the given valuation
    fn asserts(&self, val: &impl Valuation) -> Option<Literal> {
        let mut the_literal = None;
        for lit in self.literal_slice() {
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
        the_literal.copied()
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

    fn length(&self) -> usize {
        self.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn position_test_three() {
        let a = vec![
            Literal::new(1, true),
            Literal::new(2, false),
            Literal::new(4, true),
        ];
        assert_eq!(a.variable_position(1), Some(0));
        assert_eq!(a.variable_position(2), Some(1));
        assert_eq!(a.variable_position(3), None);
        assert_eq!(a.variable_position(4), Some(2));
        assert_eq!(a.variable_position(5), None);
    }

    #[test]
    fn position_test_six() {
        let a = vec![
            Literal::new(1, true),
            Literal::new(2, false),
            Literal::new(4, true),
            Literal::new(5, false),
            Literal::new(7, true),
        ];
        assert_eq!(a.variable_position(1), Some(0));
        assert_eq!(a.variable_position(2), Some(1));
        assert_eq!(a.variable_position(3), None);
        assert_eq!(a.variable_position(4), Some(2));
        assert_eq!(a.variable_position(5), Some(3));
    }
}
