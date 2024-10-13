pub mod boxed_slice;
pub mod stored;
pub mod vec;

use crate::structures::{literal::Literal, valuation::Valuation, variable::Variable};

use boxed_slice::ClauseBox;
use vec::ClauseVec;

pub trait Clause {
    fn literals(&self) -> impl Iterator<Item = Literal>;

    fn as_string(&self) -> String;

    fn as_dimacs(&self, variables: &[Variable]) -> String;

    fn to_clause_vec(self) -> ClauseVec;

    fn asserts(&self, val: &impl Valuation) -> Option<Literal>;

    fn lbd(&self, variables: &[Variable]) -> usize;
}
