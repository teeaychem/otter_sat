//! Implementation of clause trait for a slice of literals.

use crate::{
    atom_cells::AtomCells,
    config::LBD,
    db::clause::db_clause::dbClause,
    structures::{atom::Atom, clause::Clause, literal::CLiteral, valuation::Valuation},
};

impl Clause for dbClause {
    fn as_dimacs(&self, zero: bool) -> String {
        self.clause().as_dimacs(zero)
    }

    fn asserts<V: Valuation>(&self, val: &V) -> Option<CLiteral> {
        self.clause().asserts(val)
    }

    fn lbd(&self, cells: &AtomCells) -> LBD {
        self.clause().lbd(cells)
    }

    fn literals(&self) -> impl std::iter::Iterator<Item = CLiteral> {
        self.clause().literals()
    }

    fn size(&self) -> usize {
        self.clause().len()
    }

    fn atoms(&self) -> impl Iterator<Item = Atom> {
        self.clause().atoms()
    }

    fn canonical(self) -> super::CClause {
        self.clause().to_vec()
    }

    fn unsatisfiable_on(&self, valuation: &impl Valuation) -> bool {
        self.clause().unsatisfiable_on(valuation)
    }

    unsafe fn unsatisfiable_on_unchecked(&self, valuation: &impl Valuation) -> bool {
        unsafe { self.clause().unsatisfiable_on_unchecked(valuation) }
    }
}
