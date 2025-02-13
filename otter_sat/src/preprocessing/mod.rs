/*!
Procedures for preprocessing formulas.
*/
use pure::set_pure;

use crate::{
    context::GenericContext,
    misc::log::targets::{self},
    types::err::{self},
};

pub mod pure;

impl<R: rand::Rng + std::default::Default> GenericContext<R> {
    /// Applies preprocessing in accordance with the configuration of the context.
    pub fn preprocess(&mut self) -> Result<(), err::PreprocessingError> {
        if self.config.preprocessing {
            match set_pure(self) {
                Ok(()) => {}
                Err(_) => {
                    log::error!(target: targets::PREPROCESSING, "Failed to set pure literals");
                    return Err(err::PreprocessingError::Pure);
                }
            };
        }
        Ok(())
    }
}
