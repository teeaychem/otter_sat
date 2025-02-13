use crate::{
    context::GenericContext,
    structures::{
        atom::Atom,
        clause::CClause,
        literal::{IntLiteral, Literal},
    },
    types::err::{self, ParseError},
};

use core::panic;
use std::io::BufRead;

#[derive(Debug, Default, PartialEq, Eq)]
pub struct ParserInfo {
    pub expected_atoms: Option<usize>,
    pub expected_clauses: Option<usize>,
    pub added_atoms: usize,
    pub added_clauses: usize,
}

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
    /// let mut the_context = Context::from_config(Config::default());
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
    pub fn read_dimacs(&mut self, mut reader: impl BufRead) -> Result<ParserInfo, err::ErrorKind> {
        //
        let mut buffer = String::default();
        let mut clause_buffer: CClause = Vec::default();
        let mut info = ParserInfo::default();

        let mut lines = 0;

        // first phase, read until the formula begins
        'preamble_loop: loop {
            match reader.read_line(&mut buffer) {
                Ok(1) if buffer.starts_with('\n') => {
                    buffer.clear();
                    continue 'preamble_loop;
                }
                Ok(_) => lines += 1,
                Err(_) => return Err(err::ErrorKind::from(ParseError::Line(lines))),
            }

            match buffer.chars().next() {
                Some('c') => {
                    buffer.clear();
                    continue;
                }

                Some('p') => {
                    let mut problem_details = buffer.split_whitespace();
                    let atoms: usize = match problem_details.nth(2) {
                        None => return Err(err::ErrorKind::from(ParseError::ProblemSpecification)),
                        Some(string) => match string.parse() {
                            Err(_) => {
                                return Err(err::ErrorKind::from(ParseError::ProblemSpecification))
                            }

                            Ok(count) => count,
                        },
                    };

                    let clauses: usize = match problem_details.next() {
                        None => return Err(err::ErrorKind::from(ParseError::ProblemSpecification)),
                        Some(string) => match string.parse() {
                            Err(_) => {
                                return Err(err::ErrorKind::from(ParseError::ProblemSpecification))
                            }
                            Ok(count) => count,
                        },
                    };

                    buffer.clear();

                    self.ensure_atom(atoms as Atom);

                    info.expected_atoms = Some(atoms);
                    info.expected_clauses = Some(clauses);
                }

                _ => break,
            }
        }

        // second phase, read until the formula ends
        // Here, the line is advanced at the end of the loop, as the preable buffer has already set up a relevant line.
        'formula_loop: loop {
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
                                let mut clause = std::mem::take(&mut clause_buffer);
                                clause.sort_unstable();
                                clause.dedup();
                                self.add_clause(clause)?;
                            }
                            _ => {
                                let literal = match item.parse::<IntLiteral>() {
                                    Ok(int) => int.canonical(),
                                    Err(e) => panic!("{e}"),
                                };

                                clause_buffer.push(literal);
                            }
                        }
                    }
                }
            }

            buffer.clear();

            match reader.read_line(&mut buffer) {
                Ok(0) => break,
                Ok(_) => lines += 1,
                Err(_) => return Err(err::ErrorKind::from(ParseError::Line(lines))),
            }
        }

        if !clause_buffer.is_empty() {
            return Err(err::ErrorKind::from(ParseError::MissingDelimiter));
        }

        info.added_atoms = self.atom_db.count().saturating_sub(1);
        info.added_clauses = self.clause_db.current_clause_count();

        Ok(info)
    }
}

#[cfg(test)]
mod dimacs_parser_tests {

    use std::io::Write;

    use err::ErrorKind;

    use crate::{config::Config, context::Context};

    use super::*;

    #[test]
    fn bad_delimiter() {
        let mut the_context = Context::from_config(Config::default());

        let mut dimacs = vec![];
        let _ = dimacs.write(b"1  2");

        assert_eq!(
            the_context.read_dimacs(dimacs.as_slice()),
            Err(ErrorKind::Parse(ParseError::MissingDelimiter))
        );
    }

    #[test]
    fn bad_problem_spec() {
        let mut the_context = Context::from_config(Config::default());

        let mut dimacs = vec![];
        let _ = dimacs.write(
            b"
p cnf
  1  2 0",
        );

        assert_eq!(
            the_context.read_dimacs(dimacs.as_slice()),
            Err(ErrorKind::Parse(ParseError::ProblemSpecification))
        );
    }

    #[test]
    fn empty_ok() {
        let mut the_context = Context::from_config(Config::default());

        let mut dimacs = vec![];
        let _ = dimacs.write(
            b"

",
        );

        assert!(the_context.read_dimacs(dimacs.as_slice()).is_ok());
    }

    #[test]
    fn atoms_ensured() {
        let mut the_context = Context::from_config(Config::default());

        let required_atoms = 10;

        let mut dimacs = vec![];
        let _ = dimacs.write(format!("p cnf {required_atoms} 0").as_bytes());
        let _ = the_context.read_dimacs(dimacs.as_slice());

        // One extra, as the atom database always contains top.
        assert_eq!(the_context.atom_db.count(), required_atoms + 1);
    }
}
