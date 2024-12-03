mod literal;
mod literal_slice;

use crate::{config::GlueStrength, db::variable::VariableDB, structures::literal::Literal};

use super::{valuation::Valuation, variable::Variable};

pub type Clause = Vec<Literal>;

pub trait ClauseT {
    fn as_string(&self) -> String;

    fn as_dimacs(&self, variables: &VariableDB, zero: bool) -> String;

    #[allow(dead_code)]
    fn asserts(&self, val: &impl Valuation) -> Option<Literal>;

    fn lbd(&self, variable_db: &VariableDB) -> GlueStrength;

    fn literals(&self) -> impl Iterator<Item = &Literal>;

    fn size(&self) -> usize;

    fn variables(&self) -> impl Iterator<Item = Variable>;

    fn transform_to_vec(self) -> Clause;
}
