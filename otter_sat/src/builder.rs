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
        clause::{self, vClause, Clause},
        literal::{abLiteral, Literal},
    },
    types::err::{self},
};

use std::{borrow::Borrow, io::BufRead};

#[derive(Debug)]
pub enum ClauseOk {
    Tautology,
    AddedUnit,
    AddedLong,
}

/// Methods for building the context.
impl<R: rand::Rng + std::default::Default> GenericContext<R> {
    /// Returns the internal representation an atom from a string, adding the atom to the context if required.
    ///
    /// ```rust
    /// # use otter_sat::context::Context;
    /// # use otter_sat::config::Config;
    /// #
    /// let mut the_context = Context::from_config(Config::default(), None);
    /// let mut atoms = vec!["p", "-q", "r", "-r"];
    /// for atom in &atoms {
    ///     assert!(the_context.atom_from_string(&atom.to_string()).is_ok())
    /// }
    /// ```
    pub fn atom_from_string(&mut self, string: &str) -> Result<Atom, err::ParseErrorKind> {
        match self.atom_db.internal_representation(string) {
            Some(atom) => Ok(atom),
            None => {
                let the_id = self.atom_db.count() as Atom;
                self.atom_db
                    .fresh_atom(string, self.rng.gen_bool(self.config.polarity_lean));
                Ok(the_id)
            }
        }
    }

    /// Returns the internal representation of a literal from a string, adding an atom to the context if required.
    /// ```rust
    /// # use otter_sat::context::Context;
    /// # use otter_sat::config::Config;
    /// #
    /// let mut the_context = Context::from_config(Config::default(), None);
    /// let not_p = the_context.literal_from_string("-p").expect("p?");
    /// ```
    pub fn literal_from_string(&mut self, string: &str) -> Result<abLiteral, err::ParseErrorKind> {
        let trimmed_string = string.trim();
        if trimmed_string.is_empty() {
            return Err(err::ParseErrorKind::Empty);
        }
        if trimmed_string == "-" {
            return Err(err::ParseErrorKind::Negation);
        };

        let polarity = !trimmed_string.starts_with('-');

        let the_atom = match polarity {
            true => trimmed_string,
            false => &trimmed_string[1..],
        };

        // Safe, as atom_from_string takes any non-empty string, which has been established.
        let the_atom = unsafe { self.atom_from_string(the_atom).unwrap_unchecked() };
        Ok(abLiteral::fresh(the_atom, polarity))
    }

    /// Returns the internal representation a clause from a string, adding atoms to the context if required..
    ///
    /// ```rust
    /// # use otter_sat::context::Context;
    /// # use otter_sat::config::Config;
    /// # use otter_sat::dispatch::library::report::{self};
    /// #
    /// let mut the_context = Context::from_config(Config::default(), None);
    ///
    /// assert!(the_context.clause_from_string("p -q -r s").is_ok());
    /// ```
    pub fn clause_from_string(&mut self, string: &str) -> Result<vClause, err::BuildErrorKind> {
        let string_lterals = string.split_whitespace();

        let mut the_clause = vec![];

        for string_literal in string_lterals {
            let the_literal = match self.literal_from_string(string_literal) {
                Ok(literal) => literal,
                Err(e) => return Err(err::BuildErrorKind::Parse(e)),
            };

            if !the_clause.iter().any(|l| *l == the_literal) {
                the_clause.push(the_literal);
            }
        }
        Ok(the_clause)
    }

    /// Adds a clause to the context.
    ///
    /// ```rust
    /// # use otter_sat::context::Context;
    /// # use otter_sat::config::Config;
    /// # use otter_sat::dispatch::library::report::{self};
    /// #
    /// let mut the_context = Context::from_config(Config::default(), None);
    ///
    /// let a_clause = the_context.clause_from_string("p -q -r s").unwrap();
    ///
    ///  assert!(the_context.add_clause(a_clause).is_ok());
    ///  the_context.solve();
    ///  assert_eq!(the_context.report(), report::Solve::Satisfiable)
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
    pub fn add_clause(&mut self, clause: impl Clause) -> Result<ClauseOk, err::BuildErrorKind> {
        if clause.size() == 0 {
            return Err(err::BuildErrorKind::ClauseDB(
                err::ClauseDBErrorKind::EmptyClause,
            ));
        }
        let mut clause_vec = clause.canonical();

        match self.preprocess_clause(&mut clause_vec)? {
            PreprocessResult::Tautology => return Ok(ClauseOk::Tautology),
            PreprocessResult::Contradiction => return Err(err::BuildErrorKind::Unsatisfiable),
            _ => {}
        };

        match clause_vec.len() {
            0 => panic!("!"),

            1 => {
                let literal = unsafe { *clause_vec.get_unchecked(0) };

                match self.atom_db.value_of(literal.atom()) {
                    None => {
                        match self.value_and_queue(
                            literal.borrow(),
                            consequence_q::QPosition::Back,
                            0,
                        ) {
                            Ok(consequence_q::ConsequenceQueueOk::Qd) => {
                                self.record_clause(literal, clause::Source::Original, None);
                                Ok(())
                            }
                            _ => Err(err::BuildErrorKind::ClauseDB(
                                err::ClauseDBErrorKind::ImmediateConflict,
                            )),
                        }
                    }

                    Some(v) if v == literal.polarity() => {
                        // Must be at zero for an assumption, so there's nothing to do
                        if self.counters.total_decisions != 0 {
                            Err(err::BuildErrorKind::ClauseDB(
                                err::ClauseDBErrorKind::AddedUnitAfterDecision,
                            ))
                        } else {
                            Ok(())
                        }
                    }

                    Some(_) => Err(err::BuildErrorKind::ClauseDB(
                        err::ClauseDBErrorKind::ImmediateConflict,
                    )),
                };
                Ok(ClauseOk::AddedUnit)
            }

            _ => {
                if clause_vec.iter().all(|literal| {
                    self.atom_db
                        .value_of(literal.atom())
                        .is_some_and(|v| v != literal.polarity())
                }) {
                    {
                        return Err(err::BuildErrorKind::ClauseDB(
                            err::ClauseDBErrorKind::ValuationConflict,
                        ));
                    }
                }

                self.record_clause(clause_vec, clause::Source::Original, None)?;

                Ok(ClauseOk::AddedLong)
            }
        }
    }

    pub fn add_assumption(&mut self, assumption: impl Literal) -> Result<(), err::BuildErrorKind> {
        let literal = assumption;

        match self.atom_db.value_of(literal.atom()) {
            None => {
                match self.value_and_queue(literal.canonical(), consequence_q::QPosition::Back, 0) {
                    Ok(consequence_q::ConsequenceQueueOk::Qd) => {
                        self.literal_db.assumption_made(literal.canonical());
                        Ok(())
                    }
                    _ => Err(err::BuildErrorKind::ClauseDB(
                        err::ClauseDBErrorKind::ImmediateConflict,
                    )),
                }
            }

            Some(v) if v == literal.polarity() => {
                // Must be at zero for an assumption, so there's nothing to do
                if self.counters.total_decisions != 0 {
                    Err(err::BuildErrorKind::ClauseDB(
                        err::ClauseDBErrorKind::AddedUnitAfterDecision,
                    ))
                } else {
                    Ok(())
                }
            }

            Some(_) => Err(err::BuildErrorKind::ClauseDB(
                err::ClauseDBErrorKind::ImmediateConflict,
            )),
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
    ///  p  q    0
    ///  p -q    0
    /// -p  q    0
    /// -p -q    0
    ///  p  q  r 0
    /// -p  q -r 0
    ///  r -s    0
    /// ");
    ///
    /// assert!(the_context.read_dimacs(dimacs.as_slice()).is_ok());
    /// assert!(the_context.solve().is_ok());
    /// ```
    #[allow(clippy::manual_flatten, unused_labels)]
    pub fn read_dimacs(&mut self, mut reader: impl BufRead) -> Result<(), err::BuildErrorKind> {
        //

        let mut buffer = String::with_capacity(1024);
        let mut clause_buffer: vClause = Vec::default();

        let mut line_counter = 0;
        let mut clause_counter = 0;

        // first phase, read until the formula begins
        'preamble_loop: loop {
            match reader.read_line(&mut buffer) {
                Ok(0) => break,
                Ok(_) => line_counter += 1,
                Err(_) => {
                    return Err(err::BuildErrorKind::Parse(err::ParseErrorKind::Line(
                        line_counter,
                    )))
                }
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
                            return Err(err::BuildErrorKind::Parse(
                                err::ParseErrorKind::ProblemSpecification,
                            ))
                        }
                        Some(string) => match string.parse() {
                            Err(_) => {
                                return Err(err::BuildErrorKind::Parse(
                                    err::ParseErrorKind::ProblemSpecification,
                                ))
                            }
                            Ok(count) => count,
                        },
                    };

                    let clause_count: usize = match problem_details.next() {
                        None => {
                            return Err(err::BuildErrorKind::Parse(
                                err::ParseErrorKind::ProblemSpecification,
                            ))
                        }
                        Some(string) => match string.parse() {
                            Err(_) => {
                                return Err(err::BuildErrorKind::Parse(
                                    err::ParseErrorKind::ProblemSpecification,
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
                Err(_) => {
                    return Err(err::BuildErrorKind::Parse(err::ParseErrorKind::Line(
                        line_counter,
                    )))
                }
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
                                let the_literal = match self.literal_from_string(item) {
                                    Ok(literal) => literal,
                                    Err(e) => return Err(err::BuildErrorKind::Parse(e)),
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
        Ok(())
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
enum PreprocessResult {
    Tautology,
    Contradiction,
    Clause,
}

impl<R: rand::Rng + std::default::Default> GenericContext<R> {
    /// Preprocess a clause to remove proven literals and duplicate literals.
    fn preprocess_clause(
        &self,
        clause: &mut vClause,
    ) -> Result<PreprocessResult, err::BuildErrorKind> {
        let mut index = 0;
        let mut max = clause.len();
        loop {
            if index == max {
                break;
            }
            let this_l = clause[index];
            let this_n = this_l.negate();

            if clause.iter().any(|l| *l == this_n) {
                return Ok(PreprocessResult::Tautology);
            }

            if self
                .clause_db
                .all_unit_clauses()
                .any(|proven_literal| proven_literal.negate() == this_l)
            {
                clause.swap_remove(index);
                max -= 1;
            } else {
                index += 1;
            }
        }

        match clause.len() {
            0 => Ok(PreprocessResult::Contradiction),
            _ => Ok(PreprocessResult::Clause),
        }
    }
}
