mod literal;
mod literal_slice;

use crate::{config::GlueStrength, db::atom::AtomDB, structures::literal::vbLiteral};

use super::{atom::Atom, valuation::Valuation};

#[allow(non_camel_case_types)]
pub type vClause = Vec<vbLiteral>;

pub trait Clause {
    fn as_string(&self) -> String;

    fn as_dimacs(&self, atoms: &AtomDB, zero: bool) -> String;

    #[allow(dead_code)]
    fn asserts(&self, val: &impl Valuation) -> Option<vbLiteral>;

    fn lbd(&self, atom_db: &AtomDB) -> GlueStrength;

    fn literals(&self) -> impl Iterator<Item = &vbLiteral>;

    fn size(&self) -> usize;

    fn atoms(&self) -> impl Iterator<Item = Atom>;

    fn transform_to_vec(self) -> vClause;
}
