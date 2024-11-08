use crate::{
    context::stores::level::{DecisionLevel, LevelStore},
    structures::{
        literal::{Literal, LiteralSource, LiteralTrait},
        variable::Variable,
    },
};

// Information about the valuation of a variable, tied to some expectation
pub enum ValueInfo {
    NotSet,
    Match,
    Conflict,
}

use std::{borrow::Borrow, ops::DerefMut};

pub trait VariableList {
    #[allow(dead_code)]
    fn as_internal_string(&self) -> String;

    fn value_at(&self, index: usize) -> Option<bool>;

    fn value_of<L: Borrow<Literal>>(&self, literal: L) -> Option<bool>;

    #[allow(dead_code)]
    fn check_literal<L: Borrow<impl LiteralTrait>>(&self, literal: L) -> ValueInfo;

    fn set_value<L: Borrow<impl LiteralTrait>>(
        &self,
        literal: L,
        levels: &mut LevelStore,
        source: LiteralSource,
    ) -> Result<ValueInfo, ValueInfo>;

    fn slice(&self) -> &[Variable];

    fn get_unsafe(&self, index: usize) -> &Variable;
}

impl<T: ?Sized + DerefMut<Target = [Variable]>> VariableList for T {
    fn as_internal_string(&self) -> String {
        let mut the_string = String::new();
        for variable in self.iter() {
            let formatted = match variable.value() {
                Some(true) => {
                    format!(" {}", variable.id())
                }
                Some(false) => {
                    format!(" -{}", variable.id())
                }
                _ => String::default(),
            };
            the_string.push_str(formatted.as_str());
        }
        the_string
    }

    fn value_at(&self, index: usize) -> Option<bool> {
        unsafe { self.get_unchecked(index).value() }
    }

    fn value_of<L: Borrow<Literal>>(&self, literal: L) -> Option<bool> {
        unsafe { self.get_unchecked(literal.borrow().index()).value() }
    }

    fn check_literal<L: Borrow<impl LiteralTrait>>(&self, literal: L) -> ValueInfo {
        let maybe_value = unsafe { self.get_unchecked(literal.borrow().index()) };
        match maybe_value.value() {
            Some(already_set) if already_set == literal.borrow().polarity() => ValueInfo::Match,
            Some(_already_set) => ValueInfo::Conflict,
            None => ValueInfo::NotSet,
        }
    }

    // On okay reports the status of the variable *before* any actions happened
    fn set_value<L: Borrow<impl LiteralTrait>>(
        &self,
        literal: L,
        levels: &mut LevelStore,
        source: LiteralSource,
    ) -> Result<ValueInfo, ValueInfo> {
        // TODO: Fix
        // log::trace!(target: crate::log::targets::VALUATION, "Set: {}", literal.borrow());
        let variable = unsafe { self.get_unchecked(literal.borrow().index()) };
        match variable.value() {
            Some(value) if value != literal.borrow().polarity() => Err(ValueInfo::Conflict),
            Some(_value) => Ok(ValueInfo::Match),
            None => {
                variable.set_value(Some(literal.borrow().polarity()), Some(levels.index()));
                levels.record_literal(literal.borrow().canonical(), source);
                Ok(ValueInfo::NotSet)
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
