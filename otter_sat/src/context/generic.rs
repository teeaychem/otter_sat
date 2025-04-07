use std::borrow::Borrow;

use crate::{
    atom_cells::AtomCells,
    config::{Activity, Config},
    db::{
        ClauseKey, LevelIndex,
        atom::{AtomDB, AtomValue},
        clause::ClauseDB,
        trail::Trail,
        watches::Watches,
    },
    generic::index_heap::IndexHeap,
    misc::log::targets,
    reports::Report,
    structures::{
        atom::Atom,
        literal::{CLiteral, IntLiteral, Literal},
        valuation::{CValuation, Valuation},
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

    /// A current (often partial) [valuation](Valuation).
    pub valuation: CValuation,

    /// The atom database.
    /// See [db::atom](crate::db::atom) for details.
    pub atom_db: AtomDB,

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

    /// Cells indexed to atoms, containing various information.
    pub atom_cells: AtomCells,

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
        unsafe { self.set_value_unchecked(CLiteral::new(the_true, true), 0) };
    }

    /// The current valuation, as some struction which implements the valuation trait.
    pub fn valuation(&self) -> &impl Valuation {
        &self.valuation
    }

    // /// The current valuation, as a canonical [CValuation].
    // pub fn valuation_canonical(&self) -> &CValuation {
    //     &self.valuation
    // }

    pub fn value_of(&self, atom: Atom) -> Option<bool> {
        unsafe { *self.valuation.get_unchecked(atom as usize) }
    }

    /// Sets a given atom to have a given value, with a note of which decision this occurs after, if some decision has been made.
    ///
    /// # Safety
    /// No check is made on whether the atom is part of the valuation.
    pub unsafe fn set_value_unchecked(
        &mut self,
        literal: impl Borrow<CLiteral>,
        level: LevelIndex,
    ) -> AtomValue {
        let literal = literal.borrow();
        let atom = literal.atom();
        let value = literal.polarity();

        match self.value_of(atom) {
            None => unsafe {
                *self.valuation.get_unchecked_mut(atom as usize) = Some(value);
                *self.atom_db.atom_level_map.get_unchecked_mut(atom as usize) = Some(level);
                AtomValue::NotSet
            },

            Some(v) if v == value => AtomValue::Same,

            Some(_) => AtomValue::Different,
        }
    }

    /// A string representing the current valuation, using the external representation of atoms.
    pub fn valuation_string(&self) -> String {
        self.valuation()
            .atom_value_pairs()
            .filter_map(|(atom, v)| match v {
                None => None,
                Some(true) => Some(format!(" {atom}")),
                Some(false) => Some(format!("-{atom}")),
            })
            .collect::<Vec<_>>()
            .join(" ")
    }

    /// A string representing the current valuation, using [IntLiteral]s.
    pub fn valuations_ints(&self) -> Vec<IntLiteral> {
        self.valuation()
            .atom_value_pairs()
            .filter_map(|(atom, v)| match v {
                None => None,
                Some(true) => Some(atom as IntLiteral),
                Some(false) => Some(-(atom as IntLiteral)),
            })
            .collect()
    }

    /// A string representing the current valuation, using the internal representation of atoms.
    pub fn internal_valuation_string(&self) -> String {
        self.valuation()
            .atom_value_pairs()
            .filter_map(|(atom, v)| match v {
                None => None,
                Some(true) => Some((atom as isize).to_string()),
                Some(false) => Some((-(atom as isize)).to_string()),
            })
            .collect::<Vec<_>>()
            .join(" ")
    }

    /// A string representing the current valuation and the decision levels at which atoms were valued.
    /// The internal representation of atoms is used.
    pub fn internal_valuation_decision_string(&self) -> String {
        unsafe {
            self.valuation()
                .atom_value_pairs()
                .filter_map(|(atom, v)| match self.atom_db.level_unchecked(atom) {
                    None => None,
                    Some(level) => match v {
                        None => None,
                        Some(true) => Some(format!("{atom} ({level})",)),
                        Some(false) => Some(format!("-{atom} ({level})",)),
                    },
                })
                .collect::<Vec<_>>()
                .join(" ")
        }
    }

    /// Clears the value of an atom, and adds the atom to the activity heap.
    ///
    /// # Safety
    /// No check is made on whether the atom is part of the valuation.
    pub unsafe fn drop_value(&mut self, atom: Atom) {
        unsafe {
            log::trace!(target: targets::VALUATION, "Cleared atom: {atom}");
            if let Some(present) = self.value_of(atom) {
                *self
                    .atom_db
                    .previous_valuation
                    .get_unchecked_mut(atom as usize) = present;
            }
            *self.valuation.get_unchecked_mut(atom as usize) = None;
            self.atom_activity.activate(atom as usize);

            *self.atom_db.atom_level_map.get_unchecked_mut(atom as usize) = None;
        }
    }
}
