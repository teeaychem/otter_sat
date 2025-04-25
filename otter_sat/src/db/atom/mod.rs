#[doc(hidden)]
pub mod activity;

#[derive(Debug, PartialEq, Eq)]
/// The status of the valuation of an atom, relative to some known valuation.
pub enum ValuationStatus {
    /// The atom has no value.
    None,

    /// The value of the atoms is the same as the known valuation, or polarity of the literal.
    Set,

    /// The value of the atoms is not the same as the known valuation, or polarity of the literal.
    Conflict,
}
