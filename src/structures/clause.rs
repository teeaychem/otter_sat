pub mod clause_vec;
pub mod clause_box;
pub mod stored_clause;

use crate::structures::{
    literal::Literal,
    valuation::Valuation,
    variable::Variable,
};

use clause_vec::ClauseVec;
use clause_box::ClauseBox;

pub trait Clause {
    fn literals(&self) -> impl Iterator<Item = Literal>;

    fn as_string(&self) -> String;

    fn as_dimacs(&self, variables: &[Variable]) -> String;

    fn to_clause_vec(self) -> ClauseVec;

    fn asserts(&self, val: &impl Valuation) -> Option<Literal>;

    fn lbd(&self, variables: &[Variable]) -> usize;
}
