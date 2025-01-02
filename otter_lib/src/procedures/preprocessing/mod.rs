//! Procedures for preprocessing formulas.
use pure::set_pure;

use crate::{
    context::Context,
    misc::log::targets::{self},
    types::err::{self},
};

pub mod pure;

impl Context {
    /// Applies preprocessing in accordance with the configuration of the context.
    pub fn preprocess(&mut self) -> Result<(), err::Preprocessing> {
        if self.config.switch.preprocessing {
            match set_pure(self) {
                Ok(()) => {}
                Err(_) => {
                    log::error!(target: targets::PREPROCESSING, "Failed to set pure literals");
                    return Err(err::Preprocessing::Pure);
                }
            };
        }
        Ok(())
    }
}
