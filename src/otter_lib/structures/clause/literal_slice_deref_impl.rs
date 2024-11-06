use crate::{
    config::GlueStrength,
    context::stores::variable::VariableStore,
    structures::{clause::Clause, literal::Literal, variable::list::VariableList},
};

use std::ops::Deref;

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

    fn as_dimacs(&self, variables: &VariableStore) -> String {
        let mut the_string = String::new();
        for literal in self.literal_slice() {
            let the_represenetation = match literal.polarity() {
                true => format!("{} ", variables.external_name(literal.index())),
                false => format!("-{} ", variables.external_name(literal.index())),
            };
            the_string.push_str(the_represenetation.as_str());
        }
        the_string += "0";
        the_string
    }

    /// Returns the literal asserted by the clause on the given valuation
    fn asserts(&self, val: &impl VariableList) -> Option<Literal> {
        let mut the_literal = None;
        for lit in self.literal_slice() {
            if let Some(existing_val) = val.value_of(lit) {
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
    fn lbd(&self, variables: &impl VariableList) -> GlueStrength {
        let mut decision_levels = self
            .iter()
            .map(|literal| variables.get_unsafe(literal.index()).decision_level())
            .collect::<Vec<_>>();
        decision_levels.sort_unstable();
        decision_levels.dedup();
        decision_levels.len() as GlueStrength
    }

    fn length(&self) -> usize {
        self.len()
    }
}

#[cfg(test)]
mod test {
    use crate::structures::literal::Literal;

    #[test]
    fn aaa() {
        let a = &Literal::new(1, true);
        let b = &Literal::new(1, true);
        assert!(&a == &b);
    }
}
