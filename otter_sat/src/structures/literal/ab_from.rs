use crate::structures::atom::Atom;

use super::{abLiteral, Literal};

impl From<i16> for abLiteral {
    fn from(value: i16) -> Self {
        abLiteral::new(value.unsigned_abs() as Atom, value.is_positive())
    }
}

impl From<i32> for abLiteral {
    fn from(value: i32) -> Self {
        abLiteral::new(value.unsigned_abs(), value.is_positive())
    }
}

impl TryFrom<i64> for abLiteral {
    type Error = ();

    fn try_from(value: i64) -> Result<Self, Self::Error> {
        let atom = value.unsigned_abs();
        if atom < Atom::MAX.into() {
            Ok(abLiteral::new(atom as Atom, value.is_positive()))
        } else {
            Err(())
        }
    }
}

impl TryFrom<isize> for abLiteral {
    type Error = ();

    fn try_from(value: isize) -> Result<Self, Self::Error> {
        let atom = value.unsigned_abs();
        if Atom::MAX.try_into().is_ok_and(|max| atom < max) {
            Ok(abLiteral::new(atom as Atom, value.is_positive()))
        } else {
            Err(())
        }
    }
}
