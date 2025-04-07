use crate::{
    config::Config,
    db::{ClauseKey, atom::AtomDB, clause::ClauseDB, watches::Watches},
    reports::Report,
    resolution_buffer::ResolutionBuffer,
    structures::{
        atom::Atom,
        literal::{CLiteral, Literal},
    },
    types::err::ErrorKind,
};

use super::{ContextState, Counters, callbacks::CallbackTerminate};

/// A generic context, parameratised to a source of randomness.
///
/// Requires a source of [rng](rand::Rng) which (also) implements [Default].
///
/// [Default] is used in calls [make_decision](GenericContext::make_decision) to appease the borrow checker, and may be relaxed with a different implementation.
///
/// # Example
///
/// ```rust
/// # use otter_sat::context::GenericContext;
/// # use otter_sat::generic::random::MinimalPCG32;
/// # use otter_sat::config::Config;
/// let context = GenericContext::<MinimalPCG32>::from_config(Config::default());
/// ```
pub struct GenericContext<R: rand::Rng + std::default::Default> {
    /// The configuration of a context.
    pub config: Config,

    /// Counters related to a context/solve.
    pub counters: Counters,

    /// The atom database.
    /// See [db::atom](crate::db::atom) for details.
    pub atom_db: AtomDB,

    /// Watch lists for each atom in the form of [WatchDB] structs, indexed by atoms in the `watch_dbs` field.
    pub watch_dbs: Watches,

    /// The clause database.
    /// See [db::clause](crate::db::clause) for details.
    pub clause_db: ClauseDB,

    /// The status of the context.
    pub state: ContextState,

    /// The source of rng.
    pub rng: R,

    /// A buffer for resolution
    pub resolution_buffer: ResolutionBuffer,

    /// Terminates procedures, if true.
    pub(super) callback_terminate: Option<Box<CallbackTerminate>>,
}

impl<R: rand::Rng + std::default::Default> GenericContext<R> {
    /// A report on the state of the context.
    pub fn report(&self) -> Report {
        use crate::context::ContextState;
        match self.state {
            ContextState::Configuration | ContextState::Input | ContextState::Solving => {
                Report::Unknown
            }
            ContextState::Satisfiable => Report::Satisfiable,
            ContextState::Unsatisfiable(_) => Report::Unsatisfiable,
        }
    }

    /// The clause with which unsatisfiability of the context was determined by.
    pub fn unsatisfiable_clause(&self) -> Result<ClauseKey, ErrorKind> {
        match self.state {
            ContextState::Unsatisfiable(key) => Ok(key),
            _ => Err(ErrorKind::InvalidState),
        }
    }

    pub fn init(&mut self) {
        let the_true: Atom = unsafe { self.fresh_atom_fundamental(true).unwrap_unchecked() };
        unsafe {
            self.atom_db
                .set_value_unchecked(CLiteral::new(the_true, true), 0)
        };
    }
}
