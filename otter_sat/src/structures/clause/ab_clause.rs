//! Implementation of clause trait for a slice of literals.

use crate::{
    config::LBD,
    db::atom::AtomDB,
    structures::{
        atom::Atom,
        clause::Clause,
        literal::{abLiteral, cLiteral, Literal},
        valuation::Valuation,
    },
};

/// The implementation of a clause as a vector of literals.
#[allow(non_camel_case_types)]
pub type abClause = Vec<abLiteral>;

impl Clause for abClause {
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

    fn asserts(&self, val: &impl Valuation) -> Option<cLiteral> {
        let mut the_literal = None;
        for lit in self.literals() {
            if let Some(existing_val) = unsafe { val.value_of_unchecked(lit.atom()) } {
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
    fn lbd(&self, atom_db: &AtomDB) -> LBD {
        let mut decision_levels = self
            .iter()
            .map(|literal| unsafe { atom_db.decision_index_of(literal.atom()) })
            .collect::<Vec<_>>();

        decision_levels.sort_unstable();
        decision_levels.dedup();

        decision_levels.len() as LBD
    }

    fn literals(&self) -> impl std::iter::Iterator<Item = cLiteral> {
        #[cfg(feature = "boolean")]
        return self.iter().map(|literal| *literal);

        #[cfg(not(feature = "boolean"))]
        return self.iter().map(|literal| literal.canonical());
    }

    fn size(&self) -> usize {
        self.len()
    }

    fn atoms(&self) -> impl Iterator<Item = Atom> {
        self.iter().map(|literal| literal.atom())
    }

    fn canonical(self) -> super::cClause {
        #[cfg(feature = "boolean")]
        return self;

        #[cfg(not(feature = "boolean"))]
        return self
            .into_iter()
            .map(|literal| literal.canonical())
            .collect();
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
        self.literals().all(|literal| {
            valuation
                .value_of_unchecked(literal.atom())
                .is_some_and(|value| value != literal.polarity())
        })
    }
}
