use crate::{
    context::GenericContext,
    db::{atom::AssignmentStatus, watches::watch_db::WatchDB},
    structures::{
        atom::{ATOM_MAX, Atom},
        clause::{Clause, ClauseSource},
        consequence::AssignmentSource,
        literal::{CLiteral, Literal},
        valuation::Valuation,
    },
    types::err::{self, AtomDBError, ErrorKind, PreprocessingError},
};

use std::collections::HashSet;

use super::{
    ClauseOk,
    preprocess::{PreprocessingOk, preprocess_clause},
};

impl<R: rand::Rng + std::default::Default> GenericContext<R> {
    /// Returns a fresh atom.
    ///
    /// For a practical alternative, see [fresh_or_max_atom](GenericContext::fresh_or_max_atom).
    pub fn fresh_atom(&mut self) -> Result<Atom, err::AtomDBError> {
        let previous_value = self.rng.random_bool(self.config.polarity_lean.value);
        self.fresh_atom_fundamental(previous_value)
    }

    /// Returns a fresh atom, or the maximum atom.
    ///
    /// In short, a safe alternative to unwrapping the result of [fresh_atom](GenericContext::fresh_atom), by defaulting to the maximum limit of an atom.
    /// And, as exhausting the atom limit is unlikely in many applications, this may be preferred.
    ///
    /// # Panics
    /// At present, panics are not possible.
    /// However, in future this method may panic if it is not possible to obtain an atom for any reason other than exhaustion of the atom limit.
    pub fn fresh_or_max_atom(&mut self) -> Atom {
        let previous_value = self.rng.random_bool(self.config.polarity_lean.value);
        match self.fresh_atom_fundamental(previous_value) {
            Ok(atom) => atom,
            Err(err::AtomDBError::AtomsExhausted) => ATOM_MAX,
        }
    }

    /// The fundamental method for obtaining a fresh atom --- on Ok the atom is part of the language of the context.
    ///
    /// If used, all the relevant data structures are updated to support access via the atom, and the safety of each unchecked is guaranteed.
    pub fn fresh_atom_fundamental(
        &mut self,
        previous_value: bool,
    ) -> Result<Atom, err::AtomDBError> {
        let atom = match self.valuation().atom_count().try_into() {
            // Note, ATOM_MAX over Atom::Max as the former is limited by the representation of literals, if relevant.
            Ok(atom) if atom <= ATOM_MAX => atom,
            _ => {
                return Err(AtomDBError::AtomsExhausted);
            }
        };

        self.atom_activity.add(atom as usize, 1.0);

        self.watches.dbs.push(WatchDB::default());
        // self.valuation.push(None);

        self.atom_cells.grow_to_include(atom);
        let cell = self.atom_cells.get_mut(atom);
        cell.previous_value = previous_value;

        Ok(atom)
    }

    /// Ensure `atom` is present in the context --- specifically, by introducing as many atoms as required to ensure atoms form a  contiguous block: [0..`atom`].
    // As `atom` is an atom, the method is guaranteed to succeed.
    pub fn ensure_atom(&mut self, atom: Atom) {
        if self.valuation().atom_count() <= (atom as usize) {
            for _ in 0..((atom as usize) - self.valuation().atom_count()) + 1 {
                self.fresh_atom();
            }
        }
    }
}

impl<R: rand::Rng + std::default::Default> GenericContext<R> {
    /// Returns a fresh literal with value true.
    ///
    /// Alternatively, see [fresh_or_max_literal](GenericContext::fresh_or_max_literal).
    pub fn fresh_literal(&mut self) -> Result<CLiteral, err::AtomDBError> {
        let atom = self.fresh_atom()?;
        Ok(CLiteral::new(atom, true))
    }

    /// Returns a fresh literal with value true, or the maximum atom with value true.
    ///
    /// # Panics
    /// At present, panics are not possible.
    /// However, in future this method may panic if it is not possible to obtain an atom for any reason other than exhaustion of the atom limit.
    pub fn fresh_or_max_literal(&mut self) -> CLiteral {
        match self.fresh_literal() {
            Ok(literal) => literal,
            Err(err::AtomDBError::AtomsExhausted) => CLiteral::new(ATOM_MAX, true),
        }
    }

    /// Returns a vector containing `count` literals with either a fresh atom or the maximum atom and valued true.
    pub fn fresh_or_max_literals(&mut self, count: usize) -> Vec<CLiteral> {
        let mut literals = Vec::default();
        for _ in 0..count {
            literals.push(self.fresh_or_max_literal());
        }
        literals
    }
}

impl<R: rand::Rng + std::default::Default> GenericContext<R> {
    /// Adds a clause to the context, if it is compatible with the contextual valuation.
    pub fn add_clause<C: Clause>(&mut self, clause: C) -> Result<ClauseOk, err::ErrorKind> {
        if clause.size() == 0 {
            return Err(err::ErrorKind::from(err::ClauseDBError::EmptyClause));
        }
        let mut clause = clause.canonical();

        match preprocess_clause(&mut clause) {
            Ok(PreprocessingOk::Tautology) => return Ok(ClauseOk::Tautology),
            Err(PreprocessingError::Unsatisfiable) => {
                return Err(err::ErrorKind::from(err::BuildError::Unsatisfiable));
            }
            _ => {}
        };

        match clause[..] {
            [] => Err(err::ErrorKind::from(err::BuildError::EmptyClause)),

            [literal] => {
                self.ensure_atom(literal.atom());
                self.clause_db.store(
                    literal,
                    ClauseSource::Original,
                    &mut self.atom_cells,
                    &mut self.watches,
                    HashSet::default(),
                );

                match self.check_assignment(literal) {
                    AssignmentStatus::None => {
                        self.record_assignment(literal, AssignmentSource::Original);
                    }

                    AssignmentStatus::Set => {}

                    AssignmentStatus::Conflict => return Err(ErrorKind::FundamentalConflict),
                }

                Ok(ClauseOk::Added)
            }

            [..] => {
                clause.literals().for_each(|l| self.ensure_atom(l.atom()));

                self.clause_db.store(
                    clause,
                    ClauseSource::Original,
                    &mut self.atom_cells,
                    &mut self.watches,
                    HashSet::default(),
                )?;

                Ok(ClauseOk::Added)
            }
        }
    }
}
