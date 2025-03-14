//! Implementation of clause trait for a slice of literals.

use crate::{
    config::LBD,
    db::{atom::AtomDB, clause::db_clause::dbClause},
    structures::{atom::Atom, clause::Clause, literal::CLiteral, valuation::Valuation},
};

impl Clause for dbClause {
    fn as_dimacs(&self, zero: bool) -> String {
        self.clause().as_dimacs(zero)
    }

    fn asserts(&self, val: &impl Valuation) -> Option<CLiteral> {
        self.clause().asserts(val)
    }

    fn lbd(&self, atom_db: &AtomDB) -> LBD {
        self.clause().lbd(atom_db)
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
