mod literal_slice;
pub(crate) mod stored;

use crate::{
    config::GlueStrength,
    db::variable::VariableDB,
    structures::{literal::Literal, valuation::Valuation},
};

pub trait Clause {
    fn as_string(&self) -> String;

    fn as_dimacs(&self, variables: &VariableDB, zero: bool) -> String;

    #[allow(dead_code)]
    fn asserts(&self, val: &impl Valuation) -> Option<Literal>;

    fn lbd(&self, valuation: &impl Valuation) -> GlueStrength;
}
