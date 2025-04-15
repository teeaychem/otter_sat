//! Implementation of clause trait for a (single) literal.

use crate::{
    atom_cells::AtomCells,
    config::LBD,
    structures::{
        atom::Atom,
        clause::Clause,
        literal::{CLiteral, Literal},
        valuation::Valuation,
    },
};

impl Clause for CLiteral {
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
    fn asserts<V: Valuation>(&self, _val: &V) -> Option<CLiteral> {
        Some(*self)
    }

    fn lbd(&self, _cells: &AtomCells) -> LBD {
        0
    }

    fn literals(&self) -> impl Iterator<Item = CLiteral> {
        std::iter::once(self.canonical())
    }
    fn size(&self) -> usize {
        1
    }

    fn atoms(&self) -> impl Iterator<Item = crate::structures::atom::Atom> {
        std::iter::once(self.atom())
    }

    fn canonical(self) -> super::CClause {
        vec![self]
    }

    fn unsatisfiable_on(&self, valuation: &impl Valuation) -> bool {
        valuation
            .value_of(self.atom())
            .is_some_and(|value_presence| {
                value_presence.is_some_and(|value| value != self.polarity())
            })
    }

    fn literal_at(&self, index: usize) -> Option<CLiteral> {
        if index == 0 { Some(self.clone()) } else { None }
    }

    unsafe fn literal_at_unchecked(&self, index: usize) -> CLiteral {
        if index == 0 {
            self.clone()
        } else {
            panic!("!")
        }
    }

    fn atom_at(&self, index: usize) -> Option<Atom> {
        if index == 0 { Some(self.atom()) } else { None }
    }

    unsafe fn atom_at_unchecked(&self, index: usize) -> Atom {
        if index == 0 { self.atom() } else { panic!("!") }
    }

    unsafe fn unsatisfiable_on_unchecked(&self, valuation: &impl Valuation) -> bool {
        unsafe {
            valuation
                .value_of_unchecked(self.atom())
                .is_some_and(|v| v != self.polarity())
        }
    }
}
