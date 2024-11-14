mod details;
// pub use crate::structures::literal::literal_impl;

use crate::db::keys::VariableIndex;

pub type Literal = LiteralStruct;

#[derive(Clone, Copy, Debug)]
pub struct LiteralStruct {
    v_id: VariableIndex,
    polarity: bool,
}

pub trait LiteralT {
    fn new(variable_id: VariableIndex, polarity: bool) -> Self;

    fn negate(&self) -> Self;

    fn v_id(&self) -> VariableIndex;

    fn polarity(&self) -> bool;

    fn index(&self) -> usize;

    fn canonical(&self) -> Literal;
}
