//! Implementation of clause trait for a Vec of [ABLiteral]s.

use crate::{
    atom_cells::AtomCells,
    config::LBD,
    structures::{
        atom::Atom,
        clause::Clause,
        literal::{ABLiteral, CLiteral, Literal},
        valuation::Valuation,
    },
};

/// The implementation of a clause as a vector of literals.
#[allow(non_camel_case_types)]
pub type ABClause = Vec<ABLiteral>;

impl Clause for ABClause {
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
        } else {
            the_string.pop();
        }
        the_string
    }

    fn asserts<V: Valuation>(&self, val: &V) -> Option<CLiteral> {
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

    fn lbd(&self, cells: &AtomCells) -> LBD {
        let mut decision_levels = self
            .iter()
            .map(|literal| cells.level(literal.atom()))
            .collect::<Vec<_>>();

        decision_levels.sort_unstable();
        decision_levels.dedup();

        decision_levels.len() as LBD
    }

    fn literals(&self) -> impl std::iter::Iterator<Item = CLiteral> {
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

    fn canonical(self) -> super::CClause {
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

    fn literal_at(&self, index: usize) -> Option<CLiteral> {
        #[cfg(feature = "boolean")]
        return self.get(index).cloned();

        #[cfg(not(feature = "boolean"))]
        return self.get(index).map(|l| l.canonical());
    }

    unsafe fn literal_at_unchecked(&self, index: usize) -> CLiteral {
        #[cfg(feature = "boolean")]
        return unsafe { *self.get_unchecked(index) };

        #[cfg(not(feature = "boolean"))]
        return unsafe { self.get_unchecked(index).canonical() };
    }

    fn atom_at(&self, index: usize) -> Option<Atom> {
        self.get(index).map(|l| l.atom())
    }

    unsafe fn atom_at_unchecked(&self, index: usize) -> Atom {
        unsafe { self.get_unchecked(index).atom() }
    }

    unsafe fn unsatisfiable_on_unchecked(&self, valuation: &impl Valuation) -> bool {
        self.literals().all(|literal| unsafe {
            valuation
                .value_of_unchecked(literal.atom())
                .is_some_and(|value| value != literal.polarity())
        })
    }
}
