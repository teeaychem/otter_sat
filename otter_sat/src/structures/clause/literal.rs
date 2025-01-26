//! Implementation of clause trait for a (single) literal.

use crate::{
    config::LBD,
    db::atom::AtomDB,
    structures::{
        clause::Clause,
        literal::{cLiteral, Literal},
        valuation::Valuation,
    },
};

impl Clause for cLiteral {
    fn as_string(&self) -> String {
        match self.polarity() {
            true => format!(" {self}"),
            false => format!("{self}"),
        }
    }

    fn as_dimacs(&self, zero: bool) -> String {
        let mut the_string = String::new();

        let the_represenetation = match self.polarity() {
            true => format!(" {} ", self.atom()),
            false => format!("-{} ", self.atom()),
        };
        the_string.push_str(the_represenetation.as_str());

        if zero {
            the_string += "0";
            the_string
        } else {
            the_string.pop();
            the_string
        }
    }

    /// Returns the literal asserted by the clause on the given valuation
    fn asserts(&self, _val: &impl Valuation) -> Option<cLiteral> {
        Some(*self)
    }

    fn lbd(&self, _atom_db: &AtomDB) -> LBD {
        0
    }

    fn literals(&self) -> impl Iterator<Item = &cLiteral> {
        std::iter::once(self)
    }
    fn size(&self) -> usize {
        1
    }

    fn atoms(&self) -> impl Iterator<Item = crate::structures::atom::Atom> {
        std::iter::once(self.atom())
    }

    fn canonical(self) -> super::cClause {
        vec![self]
    }
}
