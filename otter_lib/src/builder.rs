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

/// Methods for building the context.
impl<R: rand::Rng + std::default::Default> GenericContext<R> {
    /// Returns the internal representation an atom from a string, adding the atom to the context if required.
    ///
    /// ```rust
    /// # use otter_lib::context::Context;
    /// # use otter_lib::config::Config;
    /// #
    /// let mut the_context = Context::from_config(Config::default(), None);
    /// let mut atoms = vec!["p", "-q", "r", "-r"];
    /// for atom in &atoms {
    ///     assert!(the_context.atom_from_string(&atom.to_string()).is_ok())
    /// }
    /// ```
    pub fn atom_from_string(&mut self, string: &str) -> Result<Atom, err::Parse> {
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
    /// # use otter_lib::context::Context;
    /// # use otter_lib::config::Config;
    /// #
    /// let mut the_context = Context::from_config(Config::default(), None);
    /// let not_p = the_context.literal_from_string("-p").expect("p?");
    /// ```
    pub fn literal_from_string(&mut self, string: &str) -> Result<abLiteral, err::Parse> {
        let trimmed_string = string.trim();
        if trimmed_string.is_empty() {
            return Err(err::Parse::Empty);
        }
        if trimmed_string == "-" {
            return Err(err::Parse::Negation);
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
    /// # use otter_lib::context::Context;
    /// # use otter_lib::config::Config;
    /// # use otter_lib::dispatch::library::report::{self};
    /// #
    /// let mut the_context = Context::from_config(Config::default(), None);
    ///
    /// assert!(the_context.clause_from_string("p -q -r s").is_ok());
    /// ```
    pub fn clause_from_string(&mut self, string: &str) -> Result<vClause, err::Build> {
        let string_lterals = string.split_whitespace();

        let mut the_clause = vec![];

        for string_literal in string_lterals {
            let the_literal = match self.literal_from_string(string_literal) {
                Ok(literal) => literal,
                Err(e) => return Err(err::Build::Parse(e)),
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
    /// # use otter_lib::context::Context;
    /// # use otter_lib::config::Config;
    /// # use otter_lib::dispatch::library::report::{self};
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
    TODO: Relax the constraints on adding a unit clause after choice.
    If the choice conflicts with the current valuation, backtracking is required.
    Otherwise, if the literal is not already recorded as a clause, it could be 'raised' to being a clause.
    Though, a naive approach may cause some issues with FRAT proofs, and other features which rely on choice level information.
     */
    pub fn add_clause(&mut self, clause: impl Clause) -> Result<(), err::Build> {
        if clause.size() == 0 {
            return Err(err::Build::ClauseDB(err::ClauseDB::EmptyClause));
        }
        let mut clause_vec = clause.canonical();

        self.preprocess_clause(&mut clause_vec)?;

        match clause_vec.len() {
            0 => Ok(()), // skip tautologies after preprocessing

            1 => {
                let literal = unsafe { *clause_vec.get_unchecked(0) };

                match self.atom_db.value_of(literal.atom()) {
                    None => match self.q_literal(literal.borrow()) {
                        Ok(consequence_q::Ok::Qd) => {
                            self.record_clause(literal, clause::Source::Original);
                            Ok(())
                        }
                        _ => Err(err::Build::ClauseDB(err::ClauseDB::ImmediateConflict)),
                    },

                    Some(v) if v == literal.polarity() => {
                        // Must be at zero for an assumption, so there's nothing to do
                        if self.counters.total_choices != 0 {
                            Err(err::Build::ClauseDB(err::ClauseDB::AddedUnitAfterChoice))
                        } else {
                            Ok(())
                        }
                    }

                    Some(_) => Err(err::Build::ClauseDB(err::ClauseDB::ImmediateConflict)),
                };
                Ok(())
            }

            _ => {
                self.record_clause(clause_vec, clause::Source::Original)?;

                Ok(())
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
    /// # use otter_lib::context::Context;
    /// # use otter_lib::config::Config;
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
    pub fn read_dimacs(&mut self, mut reader: impl BufRead) -> Result<(), err::Build> {
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
                Err(_) => return Err(err::Build::Parse(err::Parse::Line(line_counter))),
            }

            match buffer.chars().next() {
                Some('c') => {
                    buffer.clear();
                    continue;
                }

                Some('p') => {
                    let mut problem_details = buffer.split_whitespace();
                    let atom_count: usize = match problem_details.nth(2) {
                        None => return Err(err::Build::Parse(err::Parse::ProblemSpecification)),
                        Some(string) => match string.parse() {
                            Err(_) => {
                                return Err(err::Build::Parse(err::Parse::ProblemSpecification))
                            }
                            Ok(count) => count,
                        },
                    };

                    let clause_count: usize = match problem_details.next() {
                        None => return Err(err::Build::Parse(err::Parse::ProblemSpecification)),
                        Some(string) => match string.parse() {
                            Err(_) => {
                                return Err(err::Build::Parse(err::Parse::ProblemSpecification))
                            }
                            Ok(count) => count,
                        },
                    };

                    buffer.clear();

                    if let Some(dispatcher) = &self.dispatcher {
                        let expectation = report::Parser::Expected(atom_count, clause_count);
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
                Err(_) => return Err(err::Build::Parse(err::Parse::Line(line_counter))),
            }
            match buffer.chars().next() {
                Some('%') => break 'formula_loop,
                Some('c') => {}
                // Some('p') => {
                //     return Err(err::Build::Parse(err::Parse::MisplacedProblem(
                //         line_counter,
                //     )))
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
                                    Err(e) => return Err(err::Build::Parse(e)),
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
            let counts = report::Parser::Counts(self.atom_db.count(), clause_counter);
            dispatcher(Dispatch::Report(Report::Parser(counts)));
            let report_clauses =
                report::Parser::ContextClauses(self.clause_db.total_clause_count());
            dispatcher(Dispatch::Report(Report::Parser(report_clauses)));
        }
        Ok(())
    }

    // todo: implement this again, sometime
    // Aka. soft assumption
    // This will hold until a restart happens

    // pub fn believe(&mut self, literal: impl Borrow<Literal>) -> Result<(), err::Context> {
    //     if self.literal_db.choice_made() {
    //         return Err(err::Context::AssumptionAfterChoice);
    //     }
    //     match self.q_literal(literal.borrow()) {
    //         Ok(_) => {
    //             ???
    //             Ok(n())
    //         }
    //         Err(_) => Err(err::Context::AssumptionConflict),
    //     }
    // }
}

impl<R: rand::Rng + std::default::Default> GenericContext<R> {
    /// Preprocess a clause to remove proven literals and duplicate literals.
    fn preprocess_clause(&self, clause: &mut vClause) -> Result<(), err::Build> {
        let mut index = 0;
        let mut max = clause.len();
        loop {
            if index == max {
                break;
            }
            let this_l = clause[index];
            let this_n = this_l.negate();

            if clause.iter().any(|l| *l == this_n) {
                clause.clear();
                return Ok(());
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

        Ok(())
    }
}
