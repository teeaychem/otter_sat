mod details;

use crate::{db::variable::VariableDB, structures::variable::Variable};

#[derive(Clone, Copy, Debug)]
pub struct Literal {
    variable: Variable,
    polarity: bool,
}

pub trait LiteralT {
    fn new(variable_id: Variable, polarity: bool) -> Self;

    fn negate(&self) -> Self;

    fn var(&self) -> Variable;

    fn polarity(&self) -> bool;

    fn canonical(&self) -> Literal;

    fn external_representation(&self, variable_db: &VariableDB) -> String;
}
