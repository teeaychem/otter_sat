use crate::{
    context::GenericContext,
    dispatch::{
        library::report::{self, Report},
        Dispatch,
    },
    structures::{
        atom::Atom,
        clause::CClause,
        literal::{CLiteral, Literal},
    },
    types::err::{self, ErrorKind},
};

use core::panic;
use std::{collections::HashMap, io::BufRead};

impl<R: rand::Rng + std::default::Default> GenericContext<R> {
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
