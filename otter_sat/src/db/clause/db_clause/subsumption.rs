use std::borrow::Borrow;

use crate::{
    db::atom::AtomDB,
    misc::log::targets,
    structures::literal::{CLiteral, Literal},
    types::err::{self},
};

use super::dbClause;

impl dbClause {
    /*
    For the moment subsumption does not allow subsumption to a unit clause

    TODO: FRAT adjustments
    At the moment learnt clauses are modified in place.
    For FRAT it's not clear whether id overwriting is ok.
     */
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
    /// Removes the given literal from the clause, if it exists.
    /// Requires the clause has 3 or more literals --- i.e. subsumption on unit and binary clauses returns an error.
    ///
    /// As subsumption may involve the removal of a watched literal, if `fix_watch` is set then watches will be corrected after removing the literal.
    /// Watches may be left in a corrupted state as there may be no interest in fixing them.
    /// For example,  subsumption may lead to a binary clause and the watches for the clause may be set elsewhere.
    pub fn subsume(
        &mut self,
        literal: impl Borrow<CLiteral>,
        atom_db: &mut AtomDB,
        fix_watch: bool,
    ) -> Result<usize, err::SubsumptionError> {
        if self.clause.len() < 3 {
            log::error!(target: targets::SUBSUMPTION, "Subsumption attempted on non-long clause");
            return Err(err::SubsumptionError::ShortClause);
        }

        let mut position = {
            let search = self
                .clause
                .iter()
                .position(|clause_literal| clause_literal == literal.borrow());
            match search {
                None => {
                    log::error!(target: targets::SUBSUMPTION, "Pivot not found for subsumption");
                    return Err(err::SubsumptionError::NoPivot);
                }
                Some(p) => p,
            }
        };

        let mut zero_swap = false;
        if position == 0 {
            self.clause.swap(0, self.watch_ptr);
            zero_swap = true;
            position = self.watch_ptr;
        }

        let removed = self.clause.swap_remove(position);

        // Safe, as the atom is contained in a clause, and so is surely part of the database.
        match unsafe { atom_db.unwatch_long_unchecked(removed, &self.key) } {
            Ok(()) => {}
            Err(_) => return Err(err::SubsumptionError::WatchError),
        };

        if fix_watch && position == self.watch_ptr {
            let clause_length = self.clause.len();
            self.watch_ptr = 1;
            for index in 1..clause_length {
                // Safe, as index is the length of the clause.
                let index_literal = unsafe { self.clause.get_unchecked(index) };
                let index_value = atom_db.value_of(index_literal.atom());
                match index_value {
                    Some(value) if value != index_literal.polarity() => {}
                    _ => {
                        self.watch_ptr = index;
                        break;
                    }
                }
            }
            // Safe, by above construction.
            let watched_literal = unsafe { self.clause.get_unchecked(self.watch_ptr) };
            self.note_watch(watched_literal, atom_db);
            // TODO: Is this sufficient to uphold the required invariant?
            if zero_swap && atom_db.value_of(watched_literal.atom()).is_none() {
                self.clause.swap(0, self.watch_ptr);
            }
        }

        Ok(self.clause.len())
    }
}
