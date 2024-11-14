mod literal_slice;

use crate::{config::GlueStrength, db::variable::VariableDB, structures::literal::Literal};

pub trait Clause {
    fn as_string(&self) -> String;

    fn as_dimacs(&self, variables: &VariableDB, zero: bool) -> String;

    #[allow(dead_code)]
    fn asserts(&self, val: &VariableDB) -> Option<Literal>;

    fn lbd(&self, variable_db: &VariableDB) -> GlueStrength;

    fn literals(&self) -> &[Literal];
}
