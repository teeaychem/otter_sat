use crate::{
    context::GenericContext,
    structures::{
        atom::Atom,
        clause::CClause,
        literal::{IntLiteral, Literal},
        valuation::Valuation,
    },
    types::err::{self, ParseError},
};

use core::panic;
use std::io::BufRead;

/// Information about a parse.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct ParserInfo {
    /// A count of expected atoms from the problem specification, if given.
    pub expected_atoms: Option<usize>,

    /// A count of expected clauses from the problem specification, if given.
    pub expected_clauses: Option<usize>,

    /// A count of atoms added.
    pub added_atoms: usize,

    /// A count of clauses added.
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
    /// let mut ctx = Context::from_config(Config::default());
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
    /// assert!(ctx.read_dimacs(dimacs.as_slice()).is_ok());
    /// assert!(ctx.solve().is_ok());
    /// ```
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
                                return Err(err::ErrorKind::from(ParseError::ProblemSpecification));
                            }

                            Ok(count) => count,
                        },
                    };

                    let clauses: usize = match problem_details.next() {
                        None => return Err(err::ErrorKind::from(ParseError::ProblemSpecification)),
                        Some(string) => match string.parse() {
                            Err(_) => {
                                return Err(err::ErrorKind::from(ParseError::ProblemSpecification));
                            }
                            Ok(count) => count,
                        },
                    };

                    buffer.clear();

                    self.ensure_atom(atoms as Atom);

                    info.expected_atoms = Some(atoms);
                    info.expected_clauses = Some(clauses);
                }

                Some(_other_char) => break,

                None => break,
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
                _non_comment => {
                    let split_buf = buffer.split_whitespace();
                    for item in split_buf {
                        match item {
                            "0" => {
                                let mut clause = std::mem::take(&mut clause_buffer);
                                clause.sort_unstable();
                                clause.dedup();
                                self.add_clause(clause)?;
                            }

                            literal => {
                                let literal = match literal.parse::<IntLiteral>() {
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

        info.added_atoms = self.assignment().atom_count().saturating_sub(1);
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
        let mut ctx = Context::from_config(Config::default());

        let mut dimacs = vec![];
        let _ = dimacs.write(b"1  2");

        assert_eq!(
            ctx.read_dimacs(dimacs.as_slice()),
            Err(ErrorKind::Parse(ParseError::MissingDelimiter))
        );
    }

    #[test]
    fn bad_problem_spec() {
        let mut ctx = Context::from_config(Config::default());

        let mut dimacs = vec![];
        let _ = dimacs.write(
            b"
p cnf
  1  2 0",
        );

        assert_eq!(
            ctx.read_dimacs(dimacs.as_slice()),
            Err(ErrorKind::Parse(ParseError::ProblemSpecification))
        );
    }

    #[test]
    fn empty_ok() {
        let mut ctx = Context::from_config(Config::default());

        let mut dimacs = vec![];
        let _ = dimacs.write(
            b"

",
        );

        assert!(ctx.read_dimacs(dimacs.as_slice()).is_ok());
    }

    #[test]
    fn atoms_ensured() {
        let mut ctx = Context::from_config(Config::default());

        let required_atoms = 10;

        let mut dimacs = vec![];
        let _ = dimacs.write(format!("p cnf {required_atoms} 0").as_bytes());
        let _ = ctx.read_dimacs(dimacs.as_slice());

        // One extra, as the context always contains top as 0.
        assert_eq!(ctx.assignment().atom_count(), required_atoms + 1);
    }
}
