use crate::{
    config::GlueStrength,
    db::variable::VariableDB,
    structures::{
        clause::ClauseT,
        literal::{Literal, LiteralT},
        valuation::Valuation,
    },
};

use std::ops::Deref;

impl<T: Deref<Target = [Literal]>> ClauseT for T {
    fn as_string(&self) -> String {
        let mut the_string = String::default();
        for literal in self.deref() {
            the_string.push_str(format!("{literal} ").as_str());
        }
        the_string.pop();
        the_string
    }

    fn as_dimacs(&self, variables: &VariableDB, zero: bool) -> String {
        let mut the_string = String::new();
        for literal in self.deref() {
            let the_represenetation = match literal.polarity() {
                true => format!(" {} ", variables.external_representation(literal.var())),
                false => format!("-{} ", variables.external_representation(literal.var())),
            };
            the_string.push_str(the_represenetation.as_str());
        }
        if zero {
            the_string += "0";
            the_string
        } else {
            the_string.pop();
            the_string
        }
    }

    /// Returns the literal asserted by the clause on the given valuation
    fn asserts(&self, val: &impl Valuation) -> Option<Literal> {
        let mut the_literal = None;
        for lit in self.deref() {
            if let Some(existing_val) = unsafe { val.value_of(lit.var()) } {
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
    fn lbd(&self, variable_db: &VariableDB) -> GlueStrength {
        let mut decision_levels = self
            .iter()
            .map(|literal| variable_db.choice_index_of(literal.var()))
            .collect::<Vec<_>>();
        decision_levels.sort_unstable();
        decision_levels.dedup();
        decision_levels.len() as GlueStrength
    }

    fn literals(&self) -> &[Literal] {
        self
    }

    fn variables(&self) -> impl Iterator<Item = crate::structures::variable::Variable> {
        self.iter().map(|literal| literal.var())
    }
}
