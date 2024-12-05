use crate::{
    config::GlueStrength,
    db::atom::AtomDB,
    structures::{
        clause::Clause,
        literal::{vbLiteral, Literal},
        valuation::Valuation,
    },
};

impl Clause for vbLiteral {
    fn as_string(&self) -> String {
        let mut the_string = String::default();

        the_string.push_str(format!("{self}").as_str());
        the_string
    }

    fn as_dimacs(&self, atoms: &AtomDB, zero: bool) -> String {
        let mut the_string = String::new();

        let the_represenetation = match self.polarity() {
            true => format!(" {} ", atoms.external_representation(self.var())),
            false => format!("-{} ", atoms.external_representation(self.var())),
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
    fn asserts(&self, _val: &impl Valuation) -> Option<vbLiteral> {
        Some(*self)
    }

    // TODO: consider a different approach to lbd
    // e.g. an approximate measure of =2, =3, >4 can be settled much more easily
    fn lbd(&self, _atom_db: &AtomDB) -> GlueStrength {
        0
    }

    fn literals(&self) -> impl Iterator<Item = &vbLiteral> {
        std::iter::once(self)
    }
    fn size(&self) -> usize {
        1
    }

    fn atoms(&self) -> impl Iterator<Item = crate::structures::atom::Atom> {
        std::iter::once(self.var())
    }

    fn transform_to_vec(self) -> super::vClause {
        vec![self]
    }
}
