//! Values!

use super::variable::Variable;

/// The default representation of a valuation.
pub type ValuationV = Vec<Option<bool>>;

/// A valuation is something which stores some value of a variable and/or perhaps the information that the variable has no value.
pub trait Valuation {
    /// Some value of a variable under the valuation, or otherwise nothing.
    /// # Safety
    /// Implementations of `value_of` are not required to check the variable is part of the valuation.
    unsafe fn value_of(&self, variable: Variable) -> Option<bool>;

    /// An iterator over the values of a variables in the valuation, in strict, continguous, variable order.
    /// I.e. the first element is the variable '1' and then *n*th element is variable *n*.
    fn values(&self) -> impl Iterator<Item = Option<bool>>;

    /// An iterator through all (Variable, Value) pairs.
    fn vv_pairs(&self) -> impl Iterator<Item = (Variable, Option<bool>)>;

    /// An iterator through variables which have some value.
    fn valued_variables(&self) -> impl Iterator<Item = Variable>;

    /// An iterator through variables which do not have some value.
    fn unvalued_variables(&self) -> impl Iterator<Item = Variable>;
}

impl<T: std::ops::Deref<Target = [Option<bool>]>> Valuation for T {
    unsafe fn value_of(&self, variable: Variable) -> Option<bool> {
        *self.get_unchecked(variable as usize)
    }

    fn values(&self) -> impl Iterator<Item = Option<bool>> {
        self.iter().copied()
    }

    fn vv_pairs(&self) -> impl Iterator<Item = (Variable, Option<bool>)> {
        self.iter()
            .enumerate()
            .map(|(var, val)| (var as Variable, *val))
    }

    fn valued_variables(&self) -> impl Iterator<Item = Variable> {
        self.iter().enumerate().filter_map(|(var, val)| match val {
            None => None,
            _ => Some(var as Variable),
        })
    }

    fn unvalued_variables(&self) -> impl Iterator<Item = Variable> {
        self.iter().enumerate().filter_map(|(var, val)| match val {
            None => Some(var as Variable),
            _ => None,
        })
    }
}
