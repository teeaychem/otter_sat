mod details;

use crate::structures::variable::Variable;

pub type Literal = LiteralStruct;

#[derive(Clone, Copy, Debug)]
pub struct LiteralStruct {
    variable: Variable,
    polarity: bool,
}

pub trait LiteralT {
    fn new(variable_id: Variable, polarity: bool) -> Self;

    fn negate(&self) -> Self;

    fn var(&self) -> Variable;

    fn polarity(&self) -> bool;

    fn canonical(&self) -> Literal;
}
