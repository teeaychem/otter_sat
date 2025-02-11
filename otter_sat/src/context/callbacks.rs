/*!
General callbacks associated with a context.

For other callbacks see:
- [crate::db::clause::callbacks]
- [crate::ipasir]

# Callback types

Callbacks may be mutable functions.
Still, information passed from the solver is non-mutable.
*/

use super::GenericContext;

pub type CallbackTerminate = dyn FnMut() -> bool;

impl<R: rand::Rng + std::default::Default> GenericContext<R> {
    pub fn set_callback_terminate(&mut self, callback: Box<CallbackTerminate>) {
        self.callback_terminate = Some(callback);
    }

    pub fn check_callback_terminate(&mut self) -> bool {
        if let Some(callback) = &mut self.callback_terminate {
            callback()
        } else {
            false
        }
    }
}
