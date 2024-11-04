mod literal_impl;
// pub use crate::structures::literal::literal_impl;

use crate::{context::stores::ClauseKey, structures::variable::VariableId};

#[derive(Clone, Copy, Debug)]
pub struct Literal {
    v_id: VariableId,
    polarity: bool,
}

/// how a literal was settled
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[allow(clippy::upper_case_acronyms)]
pub enum LiteralSource {
    Choice,                // a choice made where the alternative may make a SAT difference
    Pure, // a choice made with a guarantee that the alternative would make no SAT difference
    Analysis(ClauseKey), // the literal must be the case for SAT given some valuation
    Resolution(ClauseKey), // there was no reason to store the resolved clause
    BCP(ClauseKey),
    Missed(ClauseKey),
    Assumption,
}
