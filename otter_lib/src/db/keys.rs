pub type DecisionIndex = usize;

pub type FormulaIndex = u32;
pub type FormulaToken = u16;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ClauseKey {
    Formula(FormulaIndex),
    Binary(FormulaIndex),
    Learned(FormulaIndex, FormulaToken),
}
