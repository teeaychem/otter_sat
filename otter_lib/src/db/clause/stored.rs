use crate::{
    db::{
        atom::{
            watch_db::{WatchStatus, Watcher},
            AtomDB,
        },
        keys::ClauseKey,
    },
    misc::log::targets::{self},
    structures::{
        atom::Atom,
        clause::{vClause, Clause},
        literal::{abLiteral, Literal},
    },
};

use std::{borrow::Borrow, ops::Deref};

#[derive(Debug)]
#[allow(non_camel_case_types)]
pub struct dbClause {
    key: ClauseKey,
    clause: vClause,
    active: bool,
    last: usize,
}

impl dbClause {
    pub fn from(key: ClauseKey, clause: vClause, atoms: &mut AtomDB) -> Self {
        let mut stored_clause = Self {
            key,
            clause,
            active: true,
            last: 0,
        };

        stored_clause.initialise_watches(atoms);

        stored_clause
    }

    pub(super) const fn key(&self) -> ClauseKey {
        self.key
    }

    pub(super) fn is_active(&self) -> bool {
        self.active
    }

    pub(super) fn deactivate(&mut self) {
        self.active = false
    }
}

// Watches

impl dbClause {
    fn initialise_watches(&mut self, atoms: &mut AtomDB) {
        let clause_length = self.clause.len() - 1;

        let mut index = 0;
        let watch_a = loop {
            if index == clause_length {
                break index;
            }

            let literal = unsafe { self.clause.get_unchecked(index) };
            let literal_value = unsafe { atoms.value_of(literal.atom()) };
            match literal_value {
                None => break index,
                Some(value) if value == literal.polarity() => break index,
                Some(_) => index += 1,
            }
        };

        self.clause.swap(0, watch_a);

        self.last = 1;
        for index in 1..clause_length {
            let index_literal = unsafe { self.clause.get_unchecked(index) };
            let index_value = unsafe { atoms.value_of(index_literal.atom()) };
            match index_value {
                None => {
                    self.last = index;
                    break;
                }
                Some(value) if value == index_literal.polarity() => {
                    self.last = index;
                    break;
                }
                Some(_) => {}
            }
        }

        unsafe {
            let first = self.clause.get_unchecked(0);
            self.note_watch(first.atom(), first.polarity(), atoms);
            let last = self.clause.get_unchecked(self.last);
            self.note_watch(last.atom(), last.polarity(), atoms);
        }
    }

    unsafe fn note_watch(&self, atom: Atom, polarity: bool, atoms: &mut AtomDB) {
        match self.key {
            ClauseKey::Unit(_) => {
                panic!("attempting to interact with watches on a unit clause")
            }
            ClauseKey::Binary(_) => {
                let check_literal = if self.clause.get_unchecked(0).atom() == atom {
                    *self.clause.get_unchecked(1)
                } else {
                    *self.clause.get_unchecked(0)
                };

                atoms.add_watch_unchecked(
                    atom,
                    polarity,
                    Watcher::Binary(check_literal, self.key()),
                );
            }
            ClauseKey::Original(_) | ClauseKey::Addition(_, _) => {
                atoms.add_watch_unchecked(atom, polarity, Watcher::Clause(self.key()));
            }
        }
    }

    ///
    /// # Safety
    /// No checks on atom as index.
    #[inline(always)]
    #[allow(clippy::result_unit_err)]
    pub unsafe fn update_watch(
        &mut self,
        atom: Atom,
        atoms: &mut AtomDB,
    ) -> Result<WatchStatus, ()> {
        /*
        This will, logic issues aside, only be called on long formulas
        And, given how often it is called, checks to ensure there are no logic issues aren't worthwhile
        The assertion is commented for when needed
         */
        // assert!(self.clause.len() > 2);

        if self.clause.get_unchecked(0).atom() == atom {
            self.clause.swap(0, self.last)
        }
        /*
        This could be split into two `for` loops around the current last index.
        This would avoid the need to check whether the search pointer is equal to where the last search pointer stopped.
        Still, it seems the single loop is easier to handle for the compiler.
         */
        let last_cache = self.last;
        let clause_length = self.clause.len();
        loop {
            self.last += 1;
            if self.last == clause_length {
                self.last = 1 // skip 0
            }
            if self.last == last_cache {
                return Err(());
            }
            let last_literal = unsafe { self.clause.get_unchecked(self.last) };
            match atoms.value_of(last_literal.atom()) {
                None => {
                    self.note_watch(last_literal.atom(), last_literal.polarity(), atoms);
                    return Ok(WatchStatus::None);
                }
                Some(value) if value == last_literal.polarity() => {
                    self.note_watch(last_literal.atom(), last_literal.polarity(), atoms);
                    return Ok(WatchStatus::Witness);
                }
                Some(_) => {}
            }
        }
    }
}

// Subsumption

#[derive(Debug, Clone, Copy)]
pub enum SubsumptionError {
    ShortClause,
    NoPivot,
    WatchError,
}

impl dbClause {
    /*
    Subsumption may result in the removal of a watched literal.
    If `fix_watch` is set then watches will be corrected after removing the literal.
    Watches may be left in a corrupted state as there may be no interest in fixing them.
    For example,  subsumption may lead to a binary clause and the watches for the clause may be set elsewhere.
    (This is what was implemented when this note was written…)

    For the moment subsumption does not allow subsumption to a unit clause

    TODO: FRAT adjustments
    At the moment learnt clauses are modified in place.
    For FRAT it's not clear whether id overwriting is ok.
     */
    pub unsafe fn subsume(
        &mut self,
        literal: impl Borrow<abLiteral>,
        atom_db: &mut AtomDB,
        fix_watch: bool,
    ) -> Result<usize, SubsumptionError> {
        if self.clause.len() < 3 {
            log::error!(target: targets::SUBSUMPTION, "Subsumption attempted on non-long clause");
            return Err(SubsumptionError::ShortClause);
        }
        let mut position = {
            let search = self
                .clause
                .iter()
                .position(|clause_literal| clause_literal == literal.borrow());
            match search {
                None => {
                    log::error!(target: targets::SUBSUMPTION, "Pivot not found for subsumption");
                    return Err(SubsumptionError::NoPivot);
                }
                Some(p) => p,
            }
        };

        if position == 0 {
            self.clause.swap(0, self.last);
            position = self.last;
        }

        let removed = self.clause.swap_remove(position);

        match atom_db.remove_watch_unchecked(removed.atom(), removed.polarity(), &self.key) {
            Ok(()) => {}
            Err(_) => return Err(SubsumptionError::WatchError),
        };

        if fix_watch && position == self.last {
            let clause_length = self.clause.len();
            self.last = 1;
            for index in 1..clause_length {
                let index_literal = self.clause.get_unchecked(index);
                let index_value = atom_db.value_of(index_literal.atom());
                match index_value {
                    Some(value) if value != index_literal.polarity() => {}
                    _ => {
                        self.last = index;
                        break;
                    }
                }
            }
            let watched_literal = self.clause.get_unchecked(self.last);
            self.note_watch(watched_literal.atom(), watched_literal.polarity(), atom_db);
        }
        Ok(self.clause.len())
    }
}

impl std::fmt::Display for dbClause {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.clause.as_string())
    }
}

impl Deref for dbClause {
    type Target = [abLiteral];

    fn deref(&self) -> &Self::Target {
        &self.clause
    }
}
