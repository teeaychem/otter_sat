mod details;

use crate::{db::atom::AtomDB, structures::atom::Atom};

#[allow(non_camel_case_types)]
#[derive(Clone, Copy, Debug)]
pub struct vbLiteral {
    atom: Atom,
    polarity: bool,
}

pub trait Literal {
    fn new(atom_id: Atom, polarity: bool) -> Self;

    fn negate(&self) -> Self;

    fn var(&self) -> Atom;

    fn polarity(&self) -> bool;

    fn canonical(&self) -> vbLiteral;

    fn external_representation(&self, atom_db: &AtomDB) -> String;
}
