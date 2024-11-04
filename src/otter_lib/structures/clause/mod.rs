mod literal_slice_deref_impl;
pub mod stored;

use crate::{
    config::GlueStrength,
    context::stores::variable::VariableStore,
    structures::{literal::Literal, variable::list::VariableList},
};

pub trait Clause {
    fn as_string(&self) -> String;

    fn as_dimacs(&self, variables: &VariableStore) -> String;

    fn asserts(&self, val: &impl VariableList) -> Option<Literal>;

    fn lbd(&self, variables: &impl VariableList) -> GlueStrength;

    fn literal_slice(&self) -> &[Literal];

    fn length(&self) -> usize;
}
