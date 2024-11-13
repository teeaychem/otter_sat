mod literal_eye;
mod literal_struct;
// pub use crate::structures::literal::literal_impl;

use crate::{db::keys::ClauseKey, structures::variable::VariableId};

pub type Literal = LiteralStruct;

#[derive(Clone, Copy, Debug)]
pub struct LiteralStruct {
    v_id: VariableId,
    polarity: bool,
}

// #[repr(transparent)]
#[derive(Clone, Copy, Debug)]
pub struct LiteralEye(isize);

/// how a literal was settled
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[allow(clippy::upper_case_acronyms)]
pub enum LiteralSource {
    Choice,                // a choice made where the alternative may make a SAT difference
    Pure,                  // a choice made when the alternative would make no SAT difference
    Analysis(ClauseKey),   // the literal must be the case for SAT given some valuation
    Resolution(ClauseKey), // there was no reason to store the resolved clause
    BCP(ClauseKey),
    Missed(ClauseKey),
    Assumption,
}

pub trait LiteralTrait {
    fn new(variable_id: VariableId, polarity: bool) -> Self;

    fn negate(&self) -> Self;

    fn v_id(&self) -> VariableId;

    fn polarity(&self) -> bool;

    fn index(&self) -> usize;

    fn canonical(&self) -> Literal;
}
