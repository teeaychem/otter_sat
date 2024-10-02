pub mod clause_vec;
pub mod stored_clause;

use crate::structures::{
    literal::Literal,
    valuation::{Valuation, ValuationVec},
    variable::{Variable, VariableId},
};

use clause_vec::ClauseVec;

pub trait Clause {
    fn literals(&self) -> impl Iterator<Item = Literal>;

    fn variables(&self) -> impl Iterator<Item = VariableId>;

    fn is_sat_on(&self, valuation: &ValuationVec) -> bool;

    fn is_unsat_on(&self, valuation: &ValuationVec) -> bool;

    fn find_unit_literal<T: Valuation>(&self, valuation: &T) -> Option<Literal>;

    fn collect_choices<T: Valuation>(&self, valuation: &T) -> Option<Vec<Literal>>;

    fn as_string(&self) -> String;

    fn as_dimacs(&self, variables: &[Variable]) -> String;

    fn is_empty(&self) -> bool;

    fn as_vec(&self) -> ClauseVec;

    fn to_vec(self) -> ClauseVec;

    fn length(&self) -> usize;

    fn asserts(&self, val: &impl Valuation) -> Option<Literal>;

    fn lbd(&self, variables: &[Variable]) -> usize;

    fn find_literal_by_id(&self, id: VariableId) -> Option<Literal>;
}

pub type ClauseId = usize;
