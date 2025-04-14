use std::borrow::Borrow;

use crate::{
    atom_cells::AtomCells,
    config::{Activity, Config},
    db::{ClauseKey, atom::AssignmentStatus, clause::ClauseDB, trail::Trail, watches::Watches},
    generic::index_heap::IndexHeap,
    misc::log::targets,
    reports::Report,
    structures::{
        atom::Atom,
        consequence::AssignmentSource,
        literal::{CLiteral, IntLiteral, Literal},
        valuation::Valuation,
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

    /// Cells indexed to atoms, containing various information.
    pub atom_cells: AtomCells,

    /// An [IndexHeap] recording the activty of atoms, where any atom without a value is 'active' on the heap.
    pub atom_activity: IndexHeap<Activity>,

    /// Watch lists for each atom in the form of [WatchDB] structs, indexed by atoms in the `watch_dbs` field.
    pub watches: Watches,

    /// The assignments made, in order from initial to most recent.
    pub trail: Trail,

    /// The clause database.
    /// See [db::clause](crate::db::clause) for details.
    pub clause_db: ClauseDB,

    /// The status of the context.
    pub state: ContextState,

    /// The source of rng.
    pub rng: R,

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
        // TODO: Double check the assignment…

        // # Safety: top is the first atom created, and so ATOM_MAX will not be hit.
        let top: Atom = unsafe { self.fresh_atom_fundamental(true).unwrap_unchecked() };
        self.record_assignment(CLiteral::new(top, true), AssignmentSource::Original);
    }

    /// The current valuation, as some struction which implements the valuation trait.
    pub fn valuation(&self) -> &impl Valuation {
        &self.atom_cells
    }
}

impl<R: rand::Rng + std::default::Default> GenericContext<R> {
    /// A string representing the current valuation, using [IntLiteral]s.
    pub fn valuations_ints(&self) -> impl Iterator<Item = IntLiteral> {
        self.valuation()
            .atom_value_pairs()
            .filter_map(|(atom, v)| match v {
                None => None,
                Some(true) => Some(atom as IntLiteral),
                Some(false) => Some(-(atom as IntLiteral)),
            })
    }

    /// A string representing the current valuation, using the internal representation of atoms.
    pub fn valuation_strings(&self) -> impl Iterator<Item = String> {
        self.valuation()
            .atom_value_pairs()
            .filter_map(|(atom, v)| match v {
                None => None,
                Some(true) => Some((atom as isize).to_string()),
                Some(false) => Some((-(atom as isize)).to_string()),
            })
    }

    /// A string representing the current valuation and the decision levels at which atoms were valued.
    /// The internal representation of atoms is used.
    pub fn valuation_decision_strings(&self) -> impl Iterator<Item = String> {
        self.valuation().atom_value_pairs().filter_map(|(atom, v)| {
            match self.atom_cells.level(atom) {
                None => None,
                Some(level) => match v {
                    None => None,
                    Some(true) => Some(format!("{atom} ({level})",)),
                    Some(false) => Some(format!("-{atom} ({level})",)),
                },
            }
        })
    }
}

impl<R: rand::Rng + std::default::Default> GenericContext<R> {
    pub fn check_assignment<BLit: Borrow<CLiteral>>(&self, literal: BLit) -> AssignmentStatus {
        let literal = literal.borrow();

        match self.value_of(literal.atom()) {
            None => AssignmentStatus::None,

            Some(v) if v == literal.polarity() => AssignmentStatus::Set,

            Some(_) => AssignmentStatus::Conflict,
        }
    }

    pub fn value_of(&self, atom: Atom) -> Option<bool> {
        // # Safety: Every atom has a valuation cell
        self.atom_cells.get_cell(atom).value
    }

    /// Clears the value of an atom, and adds the atom to the activity heap.
    ///
    /// # Safety
    /// No check is made on whether the atom is part of the valuation.
    pub fn drop_value(&mut self, atom: Atom) {
        log::trace!(target: targets::VALUATION, "Cleared atom: {atom}");

        let previous_value = self.value_of(atom);
        let cell = self.atom_cells.get_cell_mut(atom);

        if let Some(value) = previous_value {
            cell.previous_value = value;
        }

        cell.value = None;
        cell.source = AssignmentSource::None;
        cell.level = None;

        self.atom_activity.activate(atom as usize);
    }
}
