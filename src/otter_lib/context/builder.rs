use crate::{
    config::Config,
    context::Context,
    structures::{
        literal::{Literal, LiteralSource, LiteralTrait},
        variable::{Variable, VariableId},
    },
    types::{
        clause::ClauseSource,
        errs::{ClauseStoreErr, ContextErr},
    },
};

use std::{borrow::Borrow, io::BufRead, path::PathBuf};

#[derive(Debug)]
pub enum BuildErr {
    UnitClauseConflict,
    AssumptionConflict,
    AssumptionDirectConflict,
    AssumptionIndirectConflict,
    Parse(ParseErr),
    OopsAllTautologies,
    ClauseStore(ClauseStoreErr),
}

#[derive(Debug)]
pub enum ParseErr {
    ProblemSpecification,
    Line(usize),
    MisplacedProblem(usize),
    NoVariable,
    NoFile,
}

impl Context {
    pub fn variable_from_string(&mut self, name: &str) -> Result<VariableId, ParseErr> {
        match self.variables.id_of(name) {
            Some(variable) => Ok(variable),
            None => {
                let the_id = self.variables.len() as VariableId;
                self.variables.add_variable(name, Variable::new(the_id));
                Ok(the_id)
            }
        }
    }

    pub fn literal_from_string(&mut self, string: &str) -> Result<Literal, ParseErr> {
        let trimmed_string = string.trim();
        if trimmed_string.is_empty() || trimmed_string == "-" {
            return Err(ParseErr::NoVariable);
        };

        let polarity = !trimmed_string.starts_with('-');

        let mut the_name = trimmed_string;
        if !polarity {
            the_name = &the_name[1..];
        }

        let the_variable = { self.variable_from_string(the_name).unwrap() };
        Ok(Literal::new(the_variable, polarity))
    }

    // Aka. soft assumption
    // This will hold until a restart happens
    pub fn believe<L: Borrow<impl LiteralTrait>>(&mut self, literal: L) -> Result<(), ContextErr> {
        if self.levels.index() != 0 {
            return Err(ContextErr::AssumptionAfterChoice);
        }

        let assumption_result = self.q_literal(literal, LiteralSource::Assumption);
        match assumption_result {
            Ok(_) => Ok(()),
            Err(_) => Err(ContextErr::AssumptionConflict),
        }
    }

    // TODO: Type hint issue
    pub fn assume<L: Borrow<impl LiteralTrait>>(&mut self, literal: L) -> Result<(), ContextErr> {
        if self.believe(literal.borrow().canonical()).is_ok() {
            self.proofs.push(literal.borrow().canonical());
            Ok(())
        } else {
            Err(ContextErr::AssumptionConflict)
        }
    }
}

impl Context {
    pub fn clause_from_string(&mut self, string: &str) -> Result<(), BuildErr> {
        let string_lterals = string.split_whitespace();
        let mut the_clause = vec![];
        for string_literal in string_lterals {
            let the_literal = match self.literal_from_string(string_literal) {
                Ok(literal) => literal,
                Err(e) => return Err(BuildErr::Parse(e)),
            };
            if !the_clause.iter().any(|l| *l == the_literal) {
                the_clause.push(the_literal);
            }
        }

        self.store_preprocessed_clause(the_clause)
    }
}

impl Context {
    pub fn store_preprocessed_clause(&mut self, clause: Vec<Literal>) -> Result<(), BuildErr> {
        match clause.len() {
            0 => Err(BuildErr::ClauseStore(ClauseStoreErr::EmptyClause)),
            1 => {
                let literal = unsafe { *clause.get_unchecked(0) };
                match self.assume(literal) {
                    Ok(_) => Ok(()),
                    Err(_e) => Err(BuildErr::AssumptionIndirectConflict),
                }
            }
            _ => {
                let mut processed_clause: Vec<Literal> = vec![];
                let mut subsumed = vec![];

                for literal in &clause {
                    if let Some(processed_literal) = processed_clause
                        .iter()
                        .find(|l| l.index() == literal.index())
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
                            .proofs
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
                        let Ok(_) = self.assume(literal.canonical()) else {
                            return Err(BuildErr::AssumptionIndirectConflict);
                        };
                    }
                    _ => match self.store_clause(clause, ClauseSource::Formula, Vec::default()) {
                        Ok(_) => {}
                        Err(e) => return Err(BuildErr::ClauseStore(e)),
                    },
                }
                Ok(())
            }
        }
    }

    #[allow(clippy::manual_flatten, unused_labels)]
    pub fn from_dimacs_file(
        file_path: &PathBuf,
        mut file_reader: impl BufRead,
        config: Config,
    ) -> Result<Self, BuildErr> {
        let mut buffer = String::with_capacity(1024);
        let mut clause_buffer: Vec<Literal> = Vec::new();

        let config_detail = config.detail;

        let mut the_context = None;
        let mut line_counter = 0;
        let mut clause_counter = 0;

        // first phase, read until the formula begins
        'preamble_loop: loop {
            match file_reader.read_line(&mut buffer) {
                Ok(0) => break,
                Ok(_) => line_counter += 1,
                Err(_) => return Err(BuildErr::Parse(ParseErr::Line(line_counter))),
            }

            match buffer.chars().next() {
                Some('c') => {
                    buffer.clear();
                    continue;
                }
                Some('p') => {
                    let mut problem_details = buffer.split_whitespace();
                    let variable_count: usize = match problem_details.nth(2) {
                        None => return Err(BuildErr::Parse(ParseErr::ProblemSpecification)),
                        Some(string) => match string.parse() {
                            Err(_) => return Err(BuildErr::Parse(ParseErr::ProblemSpecification)),
                            Ok(count) => count,
                        },
                    };

                    let clause_count: usize = match problem_details.next() {
                        None => return Err(BuildErr::Parse(ParseErr::ProblemSpecification)),
                        Some(string) => match string.parse() {
                            Err(_) => return Err(BuildErr::Parse(ParseErr::ProblemSpecification)),
                            Ok(count) => count,
                        },
                    };

                    buffer.clear();

                    if config.show_stats {
                        println!("c Parsing {:#?}", file_path);
                        if config.detail > 0 {
                            println!("c Expectation is to get {variable_count} variables and {clause_count} clauses");
                        }
                    }
                    the_context = Some(Context::with_size_hints(
                        variable_count,
                        clause_count,
                        config.clone(),
                    ));
                    break;
                }
                _ => {
                    break;
                }
            }
        }

        let mut the_context = match the_context {
            Some(context) => context,
            None => Context::default_config(config),
        };

        'formula_loop: loop {
            match file_reader.read_line(&mut buffer) {
                Ok(0) => break,
                Ok(_) => line_counter += 1,
                Err(_) => return Err(BuildErr::Parse(ParseErr::Line(line_counter))),
            }

            match buffer.chars().next() {
                Some('%') => break 'formula_loop,
                Some('c') => {}
                Some('p') => return Err(BuildErr::Parse(ParseErr::MisplacedProblem(line_counter))),
                _ => {
                    let split_buf = buffer.split_whitespace();
                    for item in split_buf {
                        match item {
                            "0" => {
                                let the_clause = clause_buffer.clone();
                                match the_context.store_preprocessed_clause(the_clause) {
                                    Ok(_) => clause_counter += 1,
                                    Err(e) => return Err(e),
                                }

                                clause_buffer.clear();
                            }
                            _ => {
                                let the_literal = match the_context.literal_from_string(item) {
                                    Ok(literal) => literal,
                                    Err(e) => return Err(BuildErr::Parse(e)),
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

        if config_detail > 0 {
            let mut message = format!(
                "c Parsing complete with {} variables and {} clauses",
                the_context.variable_count(),
                clause_counter
            );
            if config_detail > 1 {
                message.push_str(
                    format!(" ({} added to the context)", the_context.clause_count()).as_str(),
                );
            }
            println!("{message}");
        }

        if the_context.clause_count() == 0 {
            return Err(BuildErr::OopsAllTautologies);
        }

        Ok(the_context)
    }
}
