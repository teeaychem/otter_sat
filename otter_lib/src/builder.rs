use rand::Rng;

use crate::{
    context::Context,
    dispatch::{
        library::report::{self, Report},
        Dispatch,
    },
    structures::{
        clause::Clause,
        literal::{Literal, LiteralT},
        variable::Variable,
    },
    types::{
        err::{self},
        gen::{self},
    },
};

use std::{borrow::Borrow, io::BufRead};

/// Methods for building the context.
impl Context {
    pub fn variable_from_string(&mut self, name: &str) -> Result<Variable, err::Parse> {
        match self.variable_db.internal_representation(name) {
            Some(variable) => Ok(variable),
            None => {
                let the_id = self.variable_db.count() as Variable;
                self.variable_db
                    .fresh_variable(name, self.counters.rng.gen_bool(self.config.polarity_lean));
                Ok(the_id)
            }
        }
    }

    pub fn literal_from_string(&mut self, string: &str) -> Result<Literal, err::Parse> {
        let trimmed_string = string.trim();
        if trimmed_string.is_empty() || trimmed_string == "-" {
            return Err(err::Parse::NoVariable);
        };

        let polarity = !trimmed_string.starts_with('-');

        let mut the_name = trimmed_string;
        if !polarity {
            the_name = &the_name[1..];
        }

        let the_variable = { self.variable_from_string(the_name).unwrap() };
        Ok(Literal::new(the_variable, polarity))
    }

    pub fn clause_from_string(&mut self, string: &str) -> Result<Clause, err::Build> {
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

    /// The internal representation of clauses.
    ///
    /// - Empty clauses are rejected as these are equivalent to falsum, and so unsatisfiable.
    /// - Unit clause (a literal) literal database.
    /// - Clauses with two or more literals go to the clause database.
    ///
    /// This handles the variations.
    pub fn store_clause(&mut self, clause: Clause) -> Result<(), err::Build> {
        match clause.len() {
            0 => Err(err::Build::ClauseStore(err::ClauseDB::EmptyClause)),
            1 => {
                let literal = unsafe { *clause.get_unchecked(0) };
                match self.assume(literal) {
                    Ok(_) => Ok(()),
                    Err(_e) => Err(err::Build::AssumptionIndirectConflict),
                }
            }
            _ => {
                let mut processed_clause: Clause = vec![];
                let mut subsumed = vec![];

                for literal in &clause {
                    if let Some(processed_literal) =
                        processed_clause.iter().find(|l| l.var() == literal.var())
                    {
                        if processed_literal.polarity() != literal.polarity() {
                            // Skip tautologies
                            // Could be made more efficient by sorting the literals within a clause, but preference to preserve order for now
                            return Ok(());
                        }
                        // Otherwise, avoid adding the duplicate
                    } else {
                        // Though, strengthen the clause if possible
                        if !self
                            .literal_db
                            .proven_literals()
                            .iter()
                            .any(|proven_literal| &proven_literal.negate() == literal)
                        {
                            processed_clause.push(*literal)
                        } else {
                            subsumed.push(*literal)
                        }
                    }
                }

                let clause = processed_clause;

                match clause.len() {
                    0 => {} // Any empty clause before strengthening raised an error above, so this is safe to ignore
                    1 => {
                        let literal = unsafe { clause.get_unchecked(0) };
                        let Ok(_) = self.assume(literal) else {
                            return Err(err::Build::AssumptionIndirectConflict);
                        };
                    }
                    _ => match self.clause_db.store(
                        clause,
                        gen::src::Clause::Formula,
                        &mut self.variable_db,
                    ) {
                        Ok(_) => {}
                        Err(e) => return Err(err::Build::ClauseStore(e)),
                    },
                }
                Ok(())
            }
        }
    }
}

impl Context {
    // Aka. soft assumption
    // This will hold until a restart happens
    pub fn believe(&mut self, literal: impl Borrow<Literal>) -> Result<(), err::Context> {
        if self.literal_db.choice_made() {
            return Err(err::Context::AssumptionAfterChoice);
        }
        match self.q_literal(literal.borrow()) {
            Ok(_) => {
                self.literal_db
                    .record_literal(literal, gen::src::Literal::Assumption);
                Ok(())
            }
            Err(_) => Err(err::Context::AssumptionConflict),
        }
    }

    #[allow(unused_must_use)] // ???
    pub fn assume(&mut self, literal: impl Borrow<Literal>) -> Result<(), err::Context> {
        if self.literal_db.choice_made() {
            return Err(err::Context::AssumptionAfterChoice);
        }
        match self.variable_db.value_of(literal.borrow().var()) {
            None => {
                let Ok(gen::Queue::Qd) = self.q_literal(literal.borrow()) else {
                    return Err(err::Context::AssumptionConflict);
                };
                self.literal_db
                    .record_literal(literal.borrow(), gen::src::Literal::Assumption);
                // self.store_literal(literal, src::Literal::Assumption, Vec::default());
                Ok(())
            }
            Some(v) if v == literal.borrow().polarity() => {
                // Must be at zero for an assumption, so there's nothing to do
                Ok(())
            }
            Some(_) => Err(err::Context::AssumptionConflict),
        }
    }
}

impl Context {
    #[allow(clippy::manual_flatten, unused_labels)]
    pub fn read_dimacs(&mut self, mut file_reader: impl BufRead) -> Result<(), err::Build> {
        //

        let mut buffer = String::with_capacity(1024);
        let mut clause_buffer: Clause = Vec::default();

        let mut line_counter = 0;
        let mut clause_counter = 0;

        // first phase, read until the formula begins
        'preamble_loop: loop {
            match file_reader.read_line(&mut buffer) {
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
                    let variable_count: usize = match problem_details.nth(2) {
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

                    if let Some(tx) = &self.tx {
                        let expectation = report::Parser::Expected(variable_count, clause_count);
                        tx.send(Dispatch::Report(Report::Parser(expectation)));
                    }
                    break;
                }
                _ => {
                    break;
                }
            }
        }

        'formula_loop: loop {
            match file_reader.read_line(&mut buffer) {
                Ok(0) => break,
                Ok(_) => line_counter += 1,
                Err(_) => return Err(err::Build::Parse(err::Parse::Line(line_counter))),
            }
            match buffer.chars().next() {
                Some('%') => break 'formula_loop,
                Some('c') => {}
                Some('p') => {
                    return Err(err::Build::Parse(err::Parse::MisplacedProblem(
                        line_counter,
                    )))
                }
                _ => {
                    let split_buf = buffer.split_whitespace();
                    for item in split_buf {
                        match item {
                            "0" => {
                                let the_clause = clause_buffer.clone();
                                match self.store_clause(the_clause) {
                                    Ok(_) => clause_counter += 1,
                                    Err(e) => return Err(e),
                                }

                                clause_buffer.clear();
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

        if let Some(tx) = &self.tx {
            let counts = report::Parser::Counts(self.variable_db.count(), clause_counter);
            let report_clauses = report::Parser::ContextClauses(self.clause_db.clause_count());

            tx.send(Dispatch::Report(Report::Parser(counts)));
            tx.send(Dispatch::Report(Report::Parser(report_clauses)));
        }

        Ok(())
    }
}
