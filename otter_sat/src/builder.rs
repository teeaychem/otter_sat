//! Tools for building a context.

use crate::{
    context::{ContextState, GenericContext},
    db::consequence_q::{self},
    dispatch::{
        library::report::{self, Report},
        Dispatch,
    },
    structures::{
        atom::Atom,
        clause::{CClause, Clause, ClauseSource},
        literal::{CLiteral, Literal},
    },
    types::err::{self, ErrorKind, PreprocessingError},
};

use core::panic;
use std::{
    borrow::Borrow,
    collections::{HashMap, HashSet},
    io::BufRead,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ClauseOk {
    Tautology,
    Added,
}

/// Methods for building the context.
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
            Err(err::AtomDBError::AtomsExhausted) => Atom::MAX,
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

    /// Adds a clause to the context, if it is compatible with the contextual valuation.
    ///
    /// ```rust
    /// # use otter_sat::context::Context;
    /// # use otter_sat::config::Config;
    /// # use otter_sat::dispatch::library::report::{self};
    /// # use otter_sat::structures::literal::{CLiteral, Literal};
    /// #
    /// let mut the_context = Context::from_config(Config::default(), None);
    /// let p = the_context.fresh_or_max_atom();
    /// let q = the_context.fresh_or_max_atom();
    ///
    /// let clause = vec![CLiteral::new(p, true), CLiteral::new(q, false)];
    ///
    ///  assert!(the_context.add_clause(clause).is_ok());
    ///  the_context.solve();
    ///  assert_eq!(the_context.report(), report::SolveReport::Satisfiable)
    /// ```
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

    /// Reads a DIMACS file into the context.
    ///
    /// ```rust,ignore
    /// context.read_dimacs(BufReader::new(&file))?;
    /// ```
    ///
    /// ```rust
    /// # use otter_sat::context::Context;
    /// # use otter_sat::config::Config;
    /// # use std::io::Write;
    /// let mut the_context = Context::from_config(Config::default(), None);
    ///
    /// let mut dimacs = vec![];
    /// let _ = dimacs.write(b"
    ///  1  2       0
    ///  1 -2       0
    /// -1  2       0
    /// -1 -2       0
    ///  1  2  3    0
    /// -1  2 -3    0
    ///        3 -4 0
    /// ");
    ///
    /// assert!(the_context.read_dimacs(dimacs.as_slice()).is_ok());
    /// assert!(the_context.solve().is_ok());
    /// ```
    #[allow(clippy::manual_flatten, unused_labels)]
    pub fn read_dimacs(
        &mut self,
        mut reader: impl BufRead,
    ) -> Result<HashMap<isize, Atom>, err::ErrorKind> {
        //

        let mut atom_map = HashMap::<isize, Atom>::default();
        let mut buffer = String::with_capacity(1024);
        let mut clause_buffer: CClause = Vec::default();

        let mut line_counter = 0;
        let mut clause_counter = 0;

        // first phase, read until the formula begins
        'preamble_loop: loop {
            match reader.read_line(&mut buffer) {
                Ok(0) => break,
                Ok(_) => line_counter += 1,
                Err(_) => return Err(err::ErrorKind::from(err::ParseError::Line(line_counter))),
            }

            match buffer.chars().next() {
                Some('c') => {
                    buffer.clear();
                    continue;
                }

                Some('p') => {
                    let mut problem_details = buffer.split_whitespace();
                    let atom_count: usize = match problem_details.nth(2) {
                        None => {
                            return Err(err::ErrorKind::from(err::ParseError::ProblemSpecification))
                        }
                        Some(string) => match string.parse() {
                            Err(_) => {
                                return Err(err::ErrorKind::from(
                                    err::ParseError::ProblemSpecification,
                                ))
                            }
                            Ok(count) => count,
                        },
                    };

                    let clause_count: usize = match problem_details.next() {
                        None => {
                            return Err(err::ErrorKind::from(err::ParseError::ProblemSpecification))
                        }
                        Some(string) => match string.parse() {
                            Err(_) => {
                                return Err(err::ErrorKind::from(
                                    err::ParseError::ProblemSpecification,
                                ))
                            }
                            Ok(count) => count,
                        },
                    };

                    buffer.clear();

                    if let Some(dispatcher) = &self.dispatcher {
                        let expectation = report::ParserReport::Expected(atom_count, clause_count);
                        dispatcher(Dispatch::Report(Report::Parser(expectation)));
                    }
                    break;
                }

                _ => break,
            }
        }

        // second phase, read until the formula ends
        'formula_loop: loop {
            match reader.read_line(&mut buffer) {
                Ok(0) => break,
                Ok(_) => line_counter += 1,
                Err(_) => return Err(err::ErrorKind::from(err::ParseError::Line(line_counter))),
            }
            match buffer.chars().next() {
                Some('%') => break 'formula_loop,
                Some('c') => {}
                // Some('p') => {
                //     return Err(err::BuildErrorKind::Parse(err::Parse::MisplacedProblem(line_counter)))
                // }
                _ => {
                    let split_buf = buffer.split_whitespace();
                    for item in split_buf {
                        match item {
                            "0" => {
                                let the_clause = std::mem::take(&mut clause_buffer);
                                match self.add_clause(the_clause) {
                                    Ok(_) => clause_counter += 1,
                                    Err(e) => return Err(e),
                                }
                            }
                            _ => {
                                let parsed_int = match item.parse::<isize>() {
                                    Ok(int) => int,
                                    Err(e) => panic!("{e}"),
                                };
                                let the_literal = match atom_map.get(&parsed_int.abs()) {
                                    Some(atom) => CLiteral::new(*atom, parsed_int.is_positive()),
                                    None => {
                                        let fresh_atom = match self.fresh_atom() {
                                            Ok(atom) => atom,
                                            Err(_) => return Err(ErrorKind::AtomsExhausted),
                                        };
                                        atom_map.insert(parsed_int.abs(), fresh_atom);
                                        CLiteral::new(fresh_atom, parsed_int.is_positive())
                                    }
                                };

                                if !clause_buffer.iter().any(|l| *l == the_literal) {
                                    clause_buffer.push(the_literal);
                                }
                            }
                        }
                    }
                }
            }

            buffer.clear();
        }

        if let Some(dispatcher) = &self.dispatcher {
            let counts = report::ParserReport::Counts(self.atom_db.count(), clause_counter);
            dispatcher(Dispatch::Report(Report::Parser(counts)));
            let report_clauses =
                report::ParserReport::ContextClauses(self.clause_db.total_clause_count());
            dispatcher(Dispatch::Report(Report::Parser(report_clauses)));
        }
        Ok(atom_map)
    }
}

/// Primarily to distinguish the case where preprocessing results in an empty clause.
#[derive(PartialEq, Eq)]
enum PreprocessingOk {
    /// A tautology.
    Tautology,

    /// Any clause.
    Clause,
}

/// Preprocess a clause to remove duplicate literals.
fn preprocess_clause(clause: &mut CClause) -> Result<PreprocessingOk, err::PreprocessingError> {
    let mut index = 0;
    let mut max = clause.len();
    'clause_loop: loop {
        if index == max {
            break;
        }
        let literal = clause[index];

        for other_index in 0..index {
            let other_literal = clause[other_index];
            if other_literal.atom() == literal.atom() {
                if other_literal.polarity() == literal.polarity() {
                    clause.swap_remove(index);
                    max -= 1;
                    continue 'clause_loop;
                } else {
                    return Ok(PreprocessingOk::Tautology);
                }
            }
        }
        index += 1
    }

    match clause.is_empty() {
        false => Ok(PreprocessingOk::Clause),
        true => Err(PreprocessingError::Unsatisfiable),
    }
}

#[cfg(test)]
mod preprocessing_tests {
    use super::*;

    #[test]
    fn pass() {
        let p = CLiteral::new(1, true);
        let not_q = CLiteral::new(2, false);
        let r = CLiteral::new(3, true);

        let clause = vec![p, not_q, r];
        let mut processed_clause = clause.clone();
        let _ = preprocess_clause(&mut processed_clause);

        assert!(clause.eq(&processed_clause));
    }

    #[test]
    fn duplicate_removal() {
        let p = CLiteral::new(1, true);
        let not_q = CLiteral::new(2, false);
        let r = CLiteral::new(3, true);

        let clause = vec![p, not_q, r];
        let mut processed_clause = vec![p, not_q, r, r, not_q, p];
        let _ = preprocess_clause(&mut processed_clause);

        assert!(clause.eq(&processed_clause));
    }

    #[test]
    fn contradiction_error() {
        let p = CLiteral::new(1, true);
        let not_p = CLiteral::new(1, false);

        let mut clause = vec![p, not_p];
        let preprocessing_result = preprocess_clause(&mut clause);

        assert!(preprocessing_result.is_ok_and(|k| k == PreprocessingOk::Tautology));
    }
}
