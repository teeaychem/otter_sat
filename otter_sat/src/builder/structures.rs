use crate::{
    context::GenericContext,
    db::atom::AtomValue,
    structures::{
        atom::{ATOM_MAX, Atom},
        clause::{Clause, ClauseSource},
        consequence::{Assignment, AssignmentSource},
        literal::{CLiteral, Literal},
    },
    types::err::{self, ErrorKind, PreprocessingError},
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
        self.fresh_atom_specifying_previous_value(previous_value)
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
        match self.fresh_atom_specifying_previous_value(previous_value) {
            Ok(atom) => atom,
            Err(err::AtomDBError::AtomsExhausted) => ATOM_MAX,
        }
    }

    /// A fresh atom with a specified previous value.
    pub fn fresh_atom_specifying_previous_value(
        &mut self,
        previous_value: bool,
    ) -> Result<Atom, err::AtomDBError> {
        self.atom_db.fresh_atom(previous_value)
    }

    /// Ensure `atom` is present in the context.
    pub fn ensure_atom(&mut self, atom: Atom) -> Result<(), err::AtomDBError> {
        if self.atom_db.count() <= (atom as usize) {
            for _ in 0..((atom as usize) - self.atom_db.count()) + 1 {
                self.fresh_atom();
            }
        }
        Ok(())
    }
}

impl<R: rand::Rng + std::default::Default> GenericContext<R> {
    /// Returns a fresh literal with value true.
    ///
    /// For a practical alternative, see [fresh_or_max_literal](GenericContext::fresh_or_max_literal).
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
    pub fn add_clause(&mut self, clause: impl Clause) -> Result<ClauseOk, err::ErrorKind> {
        if clause.size() == 0 {
            return Err(err::ErrorKind::from(err::ClauseDBError::EmptyClause));
        }
        let mut clause_vec = clause.canonical();

        match preprocess_clause(&mut clause_vec) {
            Ok(PreprocessingOk::Tautology) => return Ok(ClauseOk::Tautology),
            Err(PreprocessingError::Unsatisfiable) => {
                return Err(err::ErrorKind::from(err::BuildError::Unsatisfiable));
            }
            _ => {}
        };

        match clause_vec[..] {
            [] => panic!("! Empty clause"),

            [literal] => {
                self.ensure_atom(literal.atom());
                self.clause_db.store(
                    literal,
                    ClauseSource::Original,
                    &mut self.atom_db,
                    HashSet::default(),
                );
                let q_result = unsafe { self.atom_db.set_value(literal, Some(0)) };
                match q_result {
                    AtomValue::NotSet => {
                        let assignment = Assignment::from(literal, AssignmentSource::Original);
                        unsafe { self.record_assignment(assignment) };
                    }

                    AtomValue::Same => {}

                    AtomValue::Different => return Err(ErrorKind::FundamentalConflict),
                }

                Ok(ClauseOk::Added)
            }

            [..] => {
                for literal in clause_vec.literals() {
                    self.ensure_atom(literal.atom());
                }

                self.clause_db.store(
                    clause_vec,
                    ClauseSource::Original,
                    &mut self.atom_db,
                    HashSet::default(),
                )?;

                Ok(ClauseOk::Added)
            }
        }
    }
}
