use crate::{
    context::{ContextState, GenericContext},
    db::consequence_q::{self},
    structures::{
        atom::{Atom, ATOM_MAX},
        clause::{Clause, ClauseSource},
        literal::{CLiteral, Literal},
    },
    types::err::{self, PreprocessingError},
};

use std::{borrow::Borrow, collections::HashSet};

use super::{
    preprocess::{preprocess_clause, PreprocessingOk},
    ClauseOk,
};

impl<R: rand::Rng + std::default::Default> GenericContext<R> {
    /// Returns a fresh atom.
    ///
    /// For a practical alternative, see [fresh_or_max_atom](GenericContext::fresh_or_max_atom).
    pub fn fresh_atom(&mut self) -> Result<Atom, err::AtomDBError> {
        let previous_value = self.rng.gen_bool(self.config.polarity_lean);
        self.re_fresh_atom(previous_value)
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
        let previous_value = self.rng.gen_bool(self.config.polarity_lean);
        match self.re_fresh_atom(previous_value) {
            Ok(atom) => atom,
            Err(err::AtomDBError::AtomsExhausted) => ATOM_MAX,
        }
    }

    pub fn re_fresh_atom(&mut self, previous_value: bool) -> Result<Atom, err::AtomDBError> {
        self.atom_db.fresh_atom(previous_value)
    }

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
                return Err(err::ErrorKind::from(err::BuildError::Unsatisfiable))
            }
            _ => {}
        };

        match clause_vec[..] {
            [] => panic!("! Empty clause"),

            [literal] => {
                self.ensure_atom(literal.atom());
                match self.atom_db.value_of(literal.atom()) {
                    None => {
                        match self.value_and_queue(
                            literal,
                            consequence_q::QPosition::Back,
                            self.literal_db.lower_limit(),
                        ) {
                            Ok(consequence_q::ConsequenceQueueOk::Qd) => {
                                let premises = HashSet::default();
                                self.clause_db.store(
                                    literal,
                                    ClauseSource::Original,
                                    &mut self.atom_db,
                                    None,
                                    premises,
                                );
                                Ok(ClauseOk::Added)
                            }
                            _ => Err(err::ErrorKind::ValuationConflict),
                        }
                    }

                    Some(v) if v == literal.polarity() => {
                        // Must be at zero for an assumption, so there's nothing to do
                        if self.counters.total_decisions != 0 {
                            Err(err::ErrorKind::from(err::ClauseDBError::DecisionMade))
                        } else {
                            Ok(ClauseOk::Added)
                        }
                    }

                    Some(_) => Err(err::ErrorKind::ValuationConflict),
                }
            }

            [..] => {
                for literal in clause_vec.literals() {
                    self.ensure_atom(literal.atom());
                }

                if unsafe { clause_vec.unsatisfiable_on_unchecked(self.atom_db.valuation()) } {
                    return Err(err::ErrorKind::ValuationConflict);
                }

                let premises = HashSet::default();
                self.clause_db.store(
                    clause_vec,
                    ClauseSource::Original,
                    &mut self.atom_db,
                    None,
                    premises,
                )?;

                Ok(ClauseOk::Added)
            }
        }
    }

    /// Adds a clause to the database, regardless of the contextual valuation.
    ///
    /// The same checks as [GenericContext::add_clause] are made, but are used to immediately sets to the state of the solver to unsatisfiable.
    pub fn add_clause_unchecked(
        &mut self,
        clause: impl Clause,
    ) -> Result<ClauseOk, err::ErrorKind> {
        if clause.size() == 0 {
            return Err(err::ErrorKind::from(err::ClauseDBError::EmptyClause));
        }
        let mut clause_vec = clause.canonical();

        match preprocess_clause(&mut clause_vec) {
            Ok(PreprocessingOk::Tautology) => return Ok(ClauseOk::Tautology),
            Err(PreprocessingError::Unsatisfiable) => {
                return Err(err::ErrorKind::from(err::BuildError::Unsatisfiable))
            }
            _ => {}
        };

        match clause_vec[..] {
            [] => panic!("! Empty clause"),

            [literal] => {
                let premises = HashSet::default();
                self.clause_db.store(
                    literal,
                    ClauseSource::Original,
                    &mut self.atom_db,
                    None,
                    premises,
                );
                match self.value_and_queue(
                    literal.borrow(),
                    consequence_q::QPosition::Back,
                    self.literal_db.lower_limit(),
                ) {
                    Ok(consequence_q::ConsequenceQueueOk::Qd) => {
                        let premises = HashSet::default();
                        self.clause_db.store(
                            literal,
                            ClauseSource::Original,
                            &mut self.atom_db,
                            None,
                            premises,
                        );
                    }
                    _ => {
                        println!("Conflict adding clause {literal}");
                        self.state = ContextState::Unsatisfiable(
                            crate::db::ClauseKey::OriginalUnit(literal),
                        );
                    }
                }

                Ok(ClauseOk::Added)
            }

            [..] => {
                let unsatisfiable =
                    unsafe { clause_vec.unsatisfiable_on_unchecked(self.atom_db.valuation()) };

                let premises = HashSet::default();
                let result = self.clause_db.store(
                    clause_vec,
                    ClauseSource::Original,
                    &mut self.atom_db,
                    None,
                    premises,
                );
                if unsatisfiable {
                    match result {
                        Ok(key) => self.state = ContextState::Unsatisfiable(key),
                        Err(_) => panic!("! Unable to store UNSAT clause"),
                    }
                }
                Ok(ClauseOk::Added)
            }
        }
    }
}
