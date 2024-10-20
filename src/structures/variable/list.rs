use crate::structures::{
    level::LevelIndex,
    literal::Literal,
    variable::{Status, Variable},
};

use std::ops::DerefMut;

pub trait VariableList {
    fn as_display_string(&self) -> String;

    fn as_internal_string(&self) -> String;

    fn polarity_of(&self, index: usize) -> Option<bool>;

    fn check_literal(&self, literal: Literal) -> Status;

    fn check_set_value(&mut self, literal: Literal) -> Result<(), Status>;

    fn set_value(&mut self, literal: Literal, level_index: LevelIndex);

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
            None => Status::NotSet,
        }
    }

    fn check_set_value(&mut self, literal: Literal) -> Result<(), Status> {
        log::trace!("Set literal: {}", literal);
        let maybe_value = unsafe { self.get_unchecked(literal.index()) };
        match maybe_value.polarity() {
            Some(value) if value != literal.polarity() => Err(Status::Conflict),
            Some(_value) => Err(Status::Match),
            None => unsafe {
                self.get_unchecked_mut(literal.index())
                    .set_polarity(Some(literal.polarity()));
                Ok(())
            },
        }
    }

    fn set_value(&mut self, literal: Literal, level_index: LevelIndex) {
        log::trace!("Set literal: {}", literal);
        unsafe {
            let variable = self.get_unchecked_mut(literal.index());
            variable.set_polarity(Some(literal.polarity()));
            variable.set_decision_level(level_index);
        }
    }

    fn retract_valuation(&mut self, index: usize) {
        log::trace!("Clear index: {index}");
        unsafe {
            let the_variable = self.get_unchecked_mut(index);
            the_variable.set_polarity(None);
            *the_variable.decision_level.get_mut() = None;
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
