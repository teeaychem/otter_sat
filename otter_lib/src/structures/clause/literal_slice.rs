//! Implementation of clause trait for a slice of literals.

use crate::{
    config::LBD,
    db::atom::AtomDB,
    structures::{
        clause::Clause,
        literal::{abLiteral, Literal},
        valuation::Valuation,
    },
};

use std::ops::Deref;

impl<T: Deref<Target = [abLiteral]>> Clause for T {
    fn as_string(&self) -> String {
        let mut the_string = String::default();
        for literal in self.deref() {
            the_string.push_str(format!("{literal} ").as_str());
        }
        the_string.pop();
        the_string
    }

    fn as_dimacs(&self, atoms: &AtomDB, zero: bool) -> String {
        let mut the_string = String::new();
        for literal in self.deref() {
            let the_represenetation = match literal.polarity() {
                true => format!(" {} ", atoms.external_representation(literal.atom())),
                false => format!("-{} ", atoms.external_representation(literal.atom())),
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

    fn asserts(&self, val: &impl Valuation) -> Option<abLiteral> {
        let mut the_literal = None;
        for lit in self.deref() {
            if let Some(existing_val) = unsafe { val.unchecked_value_of(lit.atom()) } {
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
            .map(|literal| unsafe { atom_db.choice_index_of(literal.atom()) })
            .collect::<Vec<_>>();
        decision_levels.sort_unstable();
        decision_levels.dedup();
        decision_levels.len() as LBD
    }

    fn literals(&self) -> impl Iterator<Item = &abLiteral> {
        self.iter()
    }

    fn size(&self) -> usize {
        self.len()
    }

    fn atoms(&self) -> impl Iterator<Item = crate::structures::atom::Atom> {
        self.iter().map(|literal| literal.atom())
    }

    fn canonical(self) -> super::vClause {
        self.to_vec()
    }
}
