//! Implementation of clause trait for a slice of literals.

use crate::{
    config::LBD,
    db::atom::AtomDB,
    structures::{
        atom::Atom,
        clause::Clause,
        literal::{CLiteral, IntLiteral, Literal},
        valuation::Valuation,
    },
};

/// The implementation of a clause as a vector of integers.
#[allow(non_camel_case_types)]
pub type IntClause = Vec<IntLiteral>;

impl Clause for IntClause {
    fn as_dimacs(&self, zero: bool) -> String {
        let mut the_string = String::new();
        for literal in self.literals() {
            match literal.polarity() {
                true => the_string.push_str(format!(" {literal} ").as_str()),
                false => the_string.push_str(format!("{literal} ").as_str()),
            };
        }
        if zero {
            the_string += "0";
            the_string
        } else {
            the_string.pop();
            the_string
        }
    }

    fn asserts(&self, val: &impl Valuation) -> Option<CLiteral> {
        let mut asserted_literal = None;
        for lit in self.literals() {
            if let Some(existing_val) = unsafe { val.value_of_unchecked(lit.atom()) } {
                match existing_val == lit.polarity() {
                    true => return None,
                    false => continue,
                }
            } else if asserted_literal.is_none() {
                asserted_literal = Some(Literal::canonical(&lit));
            } else {
                return None;
            }
        }
        asserted_literal
    }

    fn lbd(&self, atom_db: &AtomDB) -> LBD {
        let mut decision_levels = self
            .iter()
            .map(|literal| unsafe { atom_db.level_unchecked(literal.atom()) })
            .collect::<Vec<_>>();

        decision_levels.sort_unstable();
        decision_levels.dedup();

        decision_levels.len() as LBD
    }

    fn literals(&self) -> impl Iterator<Item = CLiteral> {
        #[cfg(feature = "boolean")]
        return self.iter().map(|l| l.canonical());

        #[cfg(not(feature = "boolean"))]
        return self.iter().copied();
    }

    fn size(&self) -> usize {
        self.len()
    }

    fn atoms(&self) -> impl Iterator<Item = Atom> {
        self.iter().map(|literal| literal.atom())
    }

    fn canonical(self) -> super::CClause {
        #[cfg(feature = "boolean")]
        return self.into_iter().map(|l| CLiteral::canonical(&l)).collect();

        #[cfg(not(feature = "boolean"))]
        return self;
    }

    fn unsatisfiable_on(&self, valuation: &impl Valuation) -> bool {
        self.literals().all(|literal| {
            valuation
                .value_of(literal.atom())
                .is_some_and(|value_presence| {
                    value_presence.is_some_and(|value| value != literal.polarity())
                })
        })
    }

    unsafe fn unsatisfiable_on_unchecked(&self, valuation: &impl Valuation) -> bool {
        self.literals().all(|literal| unsafe {
            valuation
                .value_of_unchecked(literal.atom())
                .is_some_and(|value| value != literal.polarity())
        })
    }
}
