use crate::{
    context::stores::{level::Level, variable::ValueStatus},
    structures::{
        literal::{Literal, LiteralSource},
        variable::Variable,
    },
};

use std::{borrow::Borrow, ops::DerefMut};

pub trait VariableList {
    #[allow(dead_code)]
    fn as_internal_string(&self) -> String;

    fn value_at(&self, index: usize) -> Option<bool>;

    fn value_of<L: Borrow<Literal>>(&self, literal: L) -> Option<bool>;

    #[allow(dead_code)]
    fn check_literal<L: Borrow<Literal>>(&self, literal: L) -> ValueStatus;

    fn set_value<L: Borrow<Literal>>(
        &self,
        literal: L,
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

    fn value_at(&self, index: usize) -> Option<bool> {
        unsafe { self.get_unchecked(index).value() }
    }

    fn value_of<L: Borrow<Literal>>(&self, literal: L) -> Option<bool> {
        unsafe { self.get_unchecked(literal.borrow().index()).value() }
    }

    fn check_literal<L: Borrow<Literal>>(&self, literal: L) -> ValueStatus {
        let maybe_value = unsafe { self.get_unchecked(literal.borrow().index()) };
        match maybe_value.value() {
            Some(already_set) if already_set == literal.borrow().polarity() => ValueStatus::Match,
            Some(_already_set) => ValueStatus::Conflict,
            None => ValueStatus::Set,
        }
    }

    fn set_value<L: Borrow<Literal>>(
        &self,
        literal: L,
        level: &mut Level,
        source: LiteralSource,
    ) -> Result<ValueStatus, ValueStatus> {
        log::trace!(target: crate::log::targets::VALUATION, "Set: {}", literal.borrow());
        let variable = unsafe { self.get_unchecked(literal.borrow().index()) };
        match variable.value() {
            Some(value) if value != literal.borrow().polarity() => Err(ValueStatus::Conflict),
            Some(_value) => Ok(ValueStatus::Match),
            None => {
                variable.set_value(Some(literal.borrow().polarity()), Some(level.index()));
                level.record_literal(literal.borrow(), source);
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
