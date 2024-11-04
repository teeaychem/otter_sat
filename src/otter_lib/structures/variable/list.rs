use crate::{
    context::{level::Level, stores::variable::ValueStatus},
    structures::{
        literal::{Literal, LiteralSource},
        variable::Variable,
    },
};

use std::ops::DerefMut;

pub trait VariableList {
    fn as_internal_string(&self) -> String;

    fn value_of(&self, index: usize) -> Option<bool>;

    fn check_literal(&self, literal: Literal) -> ValueStatus;

    fn set_value(
        &self,
        literal: Literal,
        level: &mut Level,
        source: LiteralSource,
    ) -> Result<ValueStatus, ValueStatus>;

    fn slice(&self) -> &[Variable];

    fn get_unsafe(&self, index: usize) -> &Variable;
}

impl<T: ?Sized + DerefMut<Target = [Variable]>> VariableList for T {
    fn as_internal_string(&self) -> String {
        let mut the_string = String::new();
        for variable in self.iter() {
            match variable.value() {
                Some(true) => {
                    the_string.push_str(format!(" {}", variable.id()).as_str());
                }
                Some(false) => {
                    the_string.push_str(format!(" -{}", variable.id()).as_str());
                }
                _ => {}
            }
        }
        the_string
    }

    fn value_of(&self, index: usize) -> Option<bool> {
        unsafe { self.get_unchecked(index).value() }
    }

    fn check_literal(&self, literal: Literal) -> ValueStatus {
        let maybe_value = unsafe { self.get_unchecked(literal.index()) };
        match maybe_value.value() {
            Some(already_set) if already_set == literal.polarity() => ValueStatus::Match,
            Some(_already_set) => ValueStatus::Conflict,
            None => ValueStatus::Set,
        }
    }

    fn set_value(
        &self,
        literal: Literal,
        level: &mut Level,
        source: LiteralSource,
    ) -> Result<ValueStatus, ValueStatus> {
        log::trace!(target: crate::log::targets::VALUATION, "Set: {}", literal);
        let variable = unsafe { self.get_unchecked(literal.index()) };
        match variable.value() {
            Some(value) if value != literal.polarity() => Err(ValueStatus::Conflict),
            Some(_value) => Ok(ValueStatus::Match),
            None => {
                variable.set_value(Some(literal.polarity()), Some(level.index()));
                level.record_literal(literal, source);
                Ok(ValueStatus::Set)
            }
        }
    }

    fn slice(&self) -> &[Variable] {
        self
    }

    #[inline(always)]
    fn get_unsafe(&self, index: usize) -> &Variable {
        unsafe { self.get_unchecked(index) }
    }
}
