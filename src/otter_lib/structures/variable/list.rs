use crate::{
    context::level::Level,
    structures::{
        literal::{Literal, LiteralSource},
        variable::{Status, Variable},
    },
};

use std::ops::DerefMut;

pub trait VariableList {
    fn as_internal_string(&self) -> String;

    fn value_of(&self, index: usize) -> Option<bool>;

    fn check_literal(&self, literal: Literal) -> Status;

    fn set_value(
        &self,
        literal: Literal,
        level: &mut Level,
        source: LiteralSource,
    ) -> Result<Status, Status>;

    fn retract_valuation(&mut self, index: usize);

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

    fn check_literal(&self, literal: Literal) -> Status {
        let maybe_value = unsafe { self.get_unchecked(literal.index()) };
        match maybe_value.value() {
            Some(already_set) if already_set == literal.polarity() => Status::Match,
            Some(_already_set) => Status::Conflict,
            None => Status::Set,
        }
    }

    fn set_value(
        &self,
        literal: Literal,
        level: &mut Level,
        source: LiteralSource,
    ) -> Result<Status, Status> {
        log::trace!("Set literal: {}", literal);
        let variable = unsafe { self.get_unchecked(literal.index()) };
        match variable.value() {
            Some(value) if value != literal.polarity() => Err(Status::Conflict),
            Some(_value) => Ok(Status::Match),
            None => {
                variable.set_value(Some(literal.polarity()), Some(level.index()));
                level.record_literal(literal, source);
                Ok(Status::Set)
            }
        }
    }

    fn retract_valuation(&mut self, index: usize) {
        log::trace!("Clear index: {index}");
        unsafe {
            self.get_unchecked_mut(index).set_value(None, None);
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
