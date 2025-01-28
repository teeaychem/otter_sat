//! Implementation of clause trait for a slice of literals.

use crate::{
    config::LBD,
    db::{atom::AtomDB, clause::db_clause::dbClause},
    structures::{atom::Atom, clause::Clause, literal::cLiteral, valuation::Valuation},
};

impl Clause for dbClause {
    fn as_string(&self) -> String {
        self.clause().as_string()
    }

    fn as_dimacs(&self, zero: bool) -> String {
        self.clause().as_dimacs(zero)
    }

    fn asserts(&self, val: &impl Valuation) -> Option<cLiteral> {
        self.clause().asserts(val)
    }

    fn lbd(&self, atom_db: &AtomDB) -> LBD {
        self.clause().lbd(atom_db)
    }

    fn literals(&self) -> impl Iterator<Item = &cLiteral> {
        self.clause().literals()
    }

    fn size(&self) -> usize {
        self.clause().len()
    }

    fn atoms(&self) -> impl Iterator<Item = Atom> {
        self.clause().atoms()
    }

    fn canonical(self) -> super::cClause {
        self.clause().to_vec()
    }

    fn unsatisfiable_on(&self, valuation: &impl Valuation) -> bool {
        self.clause().unsatisfiable_on(valuation)
    }

    unsafe fn unsatisfiable_on_unchecked(&self, valuation: &impl Valuation) -> bool {
        self.clause().unsatisfiable_on_unchecked(valuation)
    }
}
