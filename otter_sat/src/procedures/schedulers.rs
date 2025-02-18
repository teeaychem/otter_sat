/*!
Schedulers, used to interrupt a solve for some task.

These return true if an interrupt is due, and false otherwise.

For the moment, scheduling during a solve is experimental.
*/

use crate::context::GenericContext;

impl<R: rand::Rng + std::default::Default> GenericContext<R> {
    /// Returns whether it is time for a interrupt based on whether fresh conflicts are a multiple of the current luby element.
    pub fn luby_fresh_conflict_interrupt(&self) -> bool {
        self.counters.fresh_conflicts % (self.config.luby_u.value * self.counters.luby.current())
            == 0
    }

    /// Returns whether it is time for a interrupt based on whether total conflicts is multiple of the configured interval.
    pub fn conflict_total_interrupt(&self) -> bool {
        (self.counters.total_conflicts % (self.config.conflict_mod.value as usize)) == 0
    }

    /// Returns whether it is time for a interrupt based on whether total restarts is multiple of the configured interval.
    pub fn restart_interrupt(&self) -> bool {
        (self.counters.restarts % (self.config.luby_mod.value as usize)) == 0
    }
}
