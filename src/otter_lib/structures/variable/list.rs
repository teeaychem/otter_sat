use crate::structures::{
    level::Level,
    literal::{Literal, Source as LiteralSource},
    variable::{Status, Variable},
};

use std::ops::DerefMut;

pub trait VariableList {
    fn as_display_string(&self) -> String;

    fn as_internal_string(&self) -> String;

    fn polarity_of(&self, index: usize) -> Option<bool>;

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
    fn as_display_string(&self) -> String {
        self.iter()
            .map(|variable| match variable.polarity() {
                Some(true) => variable.name().to_string(),
                Some(false) => format!("-{}", variable.name()),
                _ => String::new(),
            })
            .collect::<Vec<_>>()
            .join(" ")
    }

    fn as_internal_string(&self) -> String {
        self.iter()
            .map(|variable| match variable.polarity() {
                Some(true) => format!("{}", variable.id()),
                Some(false) => format!("-{}", variable.id()),
                _ => String::new(),
            })
            .collect::<Vec<_>>()
            .join(" ")
    }

    fn polarity_of(&self, index: usize) -> Option<bool> {
        unsafe { self.get_unchecked(index).polarity() }
    }

    fn check_literal(&self, literal: Literal) -> Status {
        let maybe_value = unsafe { self.get_unchecked(literal.index()) };
        match maybe_value.polarity() {
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
        match variable.polarity() {
            Some(value) if value != literal.polarity() => Err(Status::Conflict),
            Some(_value) => Ok(Status::Match),
            None => {
                variable.set_polarity(Some(literal.polarity()), Some(level.index()));
                level.record_literal(literal, source);
                Ok(Status::Set)
            }
        }
    }

    fn retract_valuation(&mut self, index: usize) {
        log::trace!("Clear index: {index}");
        unsafe {
            self.get_unchecked_mut(index).set_polarity(None, None);
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
