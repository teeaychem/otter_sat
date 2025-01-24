//! Tools for building a context.

use crate::{
    context::GenericContext,
    db::consequence_q::{self},
    dispatch::{
        library::report::{self, Report},
        Dispatch,
    },
    structures::{
        atom::Atom,
        clause::{self, cClause, Clause},
        literal::{cLiteral, Literal},
    },
    types::err::{self, PreprocessingError},
};

use core::panic;
use std::{
    borrow::Borrow,
    collections::{BTreeSet, HashMap},
    io::BufRead,
};

#[derive(Debug)]
pub enum ClauseOk {
    Tautology,
    AddedUnit,
    AddedLong,
}

/// Methods for building the context.
impl<R: rand::Rng + std::default::Default> GenericContext<R> {
    /// Returns a fresh atom.
    pub fn fresh_atom(&mut self) -> Result<Atom, err::AtomDBError> {
        let previous_value = self.rng.gen_bool(self.config.polarity_lean);
        self.re_fresh_atom(previous_value)
    }

    pub fn re_fresh_atom(&mut self, previous_value: bool) -> Result<Atom, err::AtomDBError> {
        self.atom_db.fresh_atom(previous_value)
    }

    /// Adds a clause to the context.
    ///
    /// ```rust
    /// # use otter_sat::context::Context;
    /// # use otter_sat::config::Config;
    /// # use otter_sat::dispatch::library::report::{self};
    /// # use otter_sat::structures::literal::{abLiteral, Literal};
    /// #
    /// let mut the_context = Context::from_config(Config::default(), None);
    /// let p = the_context.fresh_atom().unwrap();
    /// let q = the_context.fresh_atom().unwrap();
    ///
    /// let clause = vec![abLiteral::fresh(p, true), abLiteral::fresh(q, false)];
    ///
    ///  assert!(the_context.add_clause(clause).is_ok());
    ///  the_context.solve();
    ///  assert_eq!(the_context.report(), report::SolveReport::Satisfiable)
    /// ```
    ///
    /// - Empty clauses are rejected as these are equivalent to falsum, and so unsatisfiable.
    /// - Unit clause (a literal) literal database.
    /// - Clauses with two or more literals go to the clause database.
    ///
    /// This handles the variations.
    /*
    TODO: Relax the constraints on adding a unit clause after a decision has been made.
    If the decision conflicts with the current valuation, backtracking is required.
    Otherwise, if the literal is not already recorded as a clause, it could be 'raised' to being a clause.
    Though, a naive approach may cause some issues with FRAT proofs, and other features which rely on decision level information.
     */
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
            [] => panic!("!"),

            [literal] => {
                match self.atom_db.value_of(literal.atom()) {
                    None => {
                        match self.value_and_queue(
                            literal.borrow(),
                            consequence_q::QPosition::Back,
                            0,
                        ) {
                            Ok(consequence_q::ConsequenceQueueOk::Qd) => {
                                let origins = BTreeSet::default();
                                self.record_clause(
                                    literal,
                                    clause::Source::Original,
                                    None,
                                    origins,
                                );
                                Ok(())
                            }
                            _ => Err(err::ErrorKind::from(err::ClauseDBError::ImmediateConflict)),
                        }
                    }

                    Some(v) if v == literal.polarity() => {
                        // Must be at zero for an assumption, so there's nothing to do
                        if self.counters.total_decisions != 0 {
                            Err(err::ErrorKind::from(
                                err::ClauseDBError::AddedUnitAfterDecision,
                            ))
                        } else {
                            Ok(())
                        }
                    }

                    Some(_) => Err(err::ErrorKind::from(err::ClauseDBError::ImmediateConflict)),
                };
                Ok(ClauseOk::AddedUnit)
            }

            [..] => {
                if clause_vec.iter().all(|literal| {
                    self.atom_db
                        .value_of(literal.atom())
                        .is_some_and(|v| v != literal.polarity())
                }) {
                    {
                        return Err(err::ErrorKind::from(err::ClauseDBError::ValuationConflict));
                    }
                }

                let origins = BTreeSet::default();
                self.record_clause(clause_vec, clause::Source::Original, None, origins)?;

                Ok(ClauseOk::AddedLong)
            }
        }
    }

    pub fn add_assumption(&mut self, assumption: impl Literal) -> Result<(), err::ErrorKind> {
        let literal = assumption;

        match self.atom_db.value_of(literal.atom()) {
            None => {
                match self.value_and_queue(literal.canonical(), consequence_q::QPosition::Back, 0) {
                    Ok(consequence_q::ConsequenceQueueOk::Qd) => {
                        self.literal_db.assumption_made(literal.canonical());
                        Ok(())
                    }
                    _ => Err(err::ErrorKind::from(err::ClauseDBError::ImmediateConflict)),
                }
            }

            Some(v) if v == literal.polarity() => {
                // Must be at zero for an assumption, so there's nothing to do
                if self.counters.total_decisions != 0 {
                    Err(err::ErrorKind::from(
                        err::ClauseDBError::AddedUnitAfterDecision,
                    ))
                } else {
                    Ok(())
                }
            }

            Some(_) => Err(err::ErrorKind::from(err::ClauseDBError::ImmediateConflict)),
        };
        Ok(())
    }

    /// Removes assumptions from a context by unbinding the value from any atom bound due to an assumption.
    pub fn remove_assumptions(&mut self) {
        for assumption in self.literal_db.assumptions() {
            // The assumption has been added to the context, so the atom surely exists
            unsafe { self.atom_db.drop_value(assumption.atom()) };
        }
        for consequence in self.literal_db.assumption_consequences() {
            // The consequence has been observed, so the atom surely exists
            unsafe {
                self.atom_db.drop_value(consequence.atom());
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
        let mut clause_buffer: cClause = Vec::default();

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
                                // let the_literal = match self.literal_from_string(item) {
                                //     Ok(literal) => literal,
                                //     Err(e) => return Err(err::BuildErrorKind::Parse(e)),
                                // };

                                let parsed_int = match item.parse::<isize>() {
                                    Ok(int) => int,
                                    Err(e) => panic!("{e}"),
                                };
                                let the_literal = match atom_map.get(&parsed_int.abs()) {
                                    Some(atom) => cLiteral::fresh(*atom, parsed_int.is_positive()),
                                    None => {
                                        let fresh_atom = self.fresh_atom().unwrap();
                                        atom_map.insert(parsed_int.abs(), fresh_atom);
                                        cLiteral::fresh(fresh_atom, parsed_int.is_positive())
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

    // todo: implement this again, sometime
    // Aka. soft assumption
    // This will hold until a restart happens

    // pub fn believe(&mut self, literal: impl Borrow<Literal>) -> Result<(), err::Context> {
    //     if self.literal_db.decision_made() {
    //         return Err(err::Context::AssumptionAfterDecision);
    //     }
    //     match self.value_and_queue(literal.borrow()) {
    //         Ok(_) => {
    //             ???
    //             Ok(n())
    //         }
    //         Err(_) => Err(err::Context::AssumptionConflict),
    //     }
    // }
}

/// Primarily to distinguish the case where preprocessing results in an empty clause.
#[derive(PartialEq, Eq)]
enum PreprocessingOk {
    Tautology,
    Clause,
}

/// Preprocess a clause to remove duplicate literals.
fn preprocess_clause(clause: &mut cClause) -> Result<PreprocessingOk, err::PreprocessingError> {
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
    // TODO: test…
    fn pass() {
        let p = cLiteral::fresh(1, true);
        let not_q = cLiteral::fresh(2, false);
        let r = cLiteral::fresh(3, true);

        let clause = vec![p, not_q, r];
        let mut processed_clause = clause.clone();
        let _ = preprocess_clause(&mut processed_clause);

        assert!(clause.eq(&processed_clause));
    }

    #[test]
    fn duplicate_removal() {
        let p = cLiteral::fresh(1, true);
        let not_q = cLiteral::fresh(2, false);
        let r = cLiteral::fresh(3, true);

        let clause = vec![p, not_q, r];
        let mut processed_clause = vec![p, not_q, r, r, not_q, p];
        let _ = preprocess_clause(&mut processed_clause);

        assert!(clause.eq(&processed_clause));
    }

    #[test]
    fn contradiction_error() {
        let p = cLiteral::fresh(1, true);
        let not_p = cLiteral::fresh(1, false);

        let mut clause = vec![p, not_p];
        let preprocessing_result = preprocess_clause(&mut clause);

        assert!(preprocessing_result.is_ok_and(|k| k == PreprocessingOk::Tautology));
    }
}
