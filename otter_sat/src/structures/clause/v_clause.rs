//! Implementation of clause trait for a slice of literals.

use crate::{
    config::LBD,
    db::atom::AtomDB,
    structures::{
        atom::Atom,
        clause::Clause,
        literal::{cLiteral, Literal},
        valuation::Valuation,
    },
};

use std::ops::Deref;

use super::vClause;

impl Clause for vClause {
    fn as_string(&self) -> String {
        let mut the_string = String::default();
        for literal in self.deref() {
            the_string.push_str(format!("{literal} ").as_str());
        }
        the_string.pop();
        the_string
    }

    fn as_dimacs(&self, zero: bool) -> String {
        let mut the_string = String::new();
        for literal in self.deref() {
            let the_represenetation = match literal.polarity() {
                true => format!(" {} ", literal.atom()),
                false => format!("-{} ", literal.atom()),
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

    fn asserts(&self, val: &impl Valuation) -> Option<cLiteral> {
        let mut the_literal = None;
        for lit in self.deref() {
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
        the_literal.copied()
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

    fn literals(&self) -> impl Iterator<Item = &cLiteral> {
        self.iter()
    }

    fn size(&self) -> usize {
        self.len()
    }

    fn atoms(&self) -> impl Iterator<Item = Atom> {
        self.iter().map(|literal| literal.atom())
    }

    fn canonical(self) -> super::cClause {
        self
    }
}
