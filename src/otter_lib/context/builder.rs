use crate::{
    config::Config,
    context::Context,
    structures::{
        clause::stored::ClauseSource,
        literal::{Literal, LiteralSource},
        variable::{list::VariableList, Variable, VariableId},
    },
};

use std::{io::BufRead, path::PathBuf};

use super::core::ContextIssue;

#[derive(Debug)]
pub enum BuildIssue {
    UnitClauseConflict,
    AssumptionConflict,
    AssumptionDirectConflict,
    AssumptionIndirectConflict,
    ClauseEmpty,
    Parse(ParseIssue),
    OopsAllTautologies,
}

#[derive(Debug)]
pub enum ParseIssue {
    ProblemSpecification,
    Line(usize),
    MisplacedProblem(usize),
    NoVariable,
    NoFile,
}

impl Context {
    pub fn literal_from_string(&mut self, string: &str) -> Result<Literal, ParseIssue> {
        let trimmed_string = string.trim();
        if trimmed_string.is_empty() || trimmed_string == "-" {
            return Err(ParseIssue::NoVariable);
        };

        let polarity = !trimmed_string.starts_with('-');

        let mut the_name = trimmed_string;
        if !polarity {
            the_name = &the_name[1..];
        }

        let the_variable = {
            match self.variables.id_of(the_name) {
                Some(variable) => variable,
                None => {
                    let the_id = self.variables.len() as VariableId;
                    self.variables.add_variable(the_name, Variable::new(the_id));
                    the_id
                }
            }
        };
        Ok(Literal::new(the_variable, polarity))
    }

    pub fn assume(&mut self, literal: Literal) -> Result<(), BuildIssue> {
        assert_eq!(self.levels.index(), 0);
        let assumption_result = self.q_literal(literal, LiteralSource::Assumption);
        match assumption_result {
            Ok(_) => Ok(()),
            Err(_) => Err(BuildIssue::AssumptionConflict),
        }
    }

    pub fn clause_from_string(&mut self, string: &str) -> Result<(), BuildIssue> {
        let string_lterals = string.split_whitespace();
        let mut the_clause = vec![];
        for string_literal in string_lterals {
            let the_literal = match self.literal_from_string(string_literal) {
                Ok(literal) => literal,
                Err(e) => return Err(BuildIssue::Parse(e)),
            };
            if !the_clause.iter().any(|l| *l == the_literal) {
                the_clause.push(the_literal);
            }
        }

        self.preprocess_and_store_clause(the_clause)
    }

    pub fn preprocess_and_store_clause(&mut self, clause: Vec<Literal>) -> Result<(), BuildIssue> {
        match clause.len() {
            0 => Err(BuildIssue::ClauseEmpty),
            1 => {
                let literal = unsafe { *clause.get_unchecked(0) };
                match self.assume(literal) {
                    Ok(_) => Ok(()),
                    Err(_e) => Err(BuildIssue::AssumptionIndirectConflict),
                }
            }
            _ => {
                // todo: temporary tautology check
                // do not add a tautology
                for literal in &clause {
                    if clause.iter().any(|l| *l == literal.negate()) {
                        return Ok(());
                    }
                }

                let mut strengthened_clause = vec![];
                let mut subsumed = vec![];

                // strengthen a clause given established assumptions and skip adding a satisfied clause
                for literal in clause {
                    match self.variables.value_of(literal.index()) {
                        None => {
                            strengthened_clause.push(literal);
                        }
                        Some(value) if value != literal.polarity() => subsumed.push(literal),
                        Some(_) => {
                            strengthened_clause.push(literal);
                            return Ok(());
                        }
                    }
                }

                match strengthened_clause.len() {
                    0 => {} // Any empty clause before strengthening raised an error above, so this is safe to ignore
                    1 => {
                        let literal = strengthened_clause[0];
                        match self.assume(literal) {
                            Ok(_) => {}
                            Err(_e) => return Err(BuildIssue::AssumptionIndirectConflict),
                        }
                    }
                    _ => {
                        match self.store_clause(
                            strengthened_clause,
                            subsumed,
                            ClauseSource::Formula,
                            None,
                        ) {
                            Ok(_) => {}
                            Err(ContextIssue::EmptyClause) => {
                                return Err(BuildIssue::ClauseEmpty);
                            }
                        }
                    }
                }
                Ok(())
            }
        }
    }

    #[allow(clippy::manual_flatten, unused_labels)]
    pub fn from_dimacs(
        file_path: &PathBuf,
        mut file_reader: impl BufRead,
        config: Config,
    ) -> Result<Self, BuildIssue> {
        let mut buffer = String::with_capacity(1024);
        let mut clause_buffer: Vec<Literal> = Vec::new();

        let mut the_context = None;
        let mut line_counter = 0;
        let mut clause_counter = 0;

        let show_stats = config.show_stats;

        // first phase, read until the formula begins
        'preamble_loop: loop {
            match file_reader.read_line(&mut buffer) {
                Ok(0) => break,
                Ok(_) => line_counter += 1,
                Err(_) => return Err(BuildIssue::Parse(ParseIssue::Line(line_counter))),
            }

            match buffer.chars().next() {
                Some('c') => {
                    buffer.clear();
                    continue;
                }
                Some('p') => {
                    let mut problem_details = buffer.split_whitespace();
                    let variable_count: usize = match problem_details.nth(2) {
                        None => return Err(BuildIssue::Parse(ParseIssue::ProblemSpecification)),
                        Some(string) => match string.parse() {
                            Err(_) => {
                                return Err(BuildIssue::Parse(ParseIssue::ProblemSpecification))
                            }
                            Ok(count) => count,
                        },
                    };

                    let clause_count: usize = match problem_details.next() {
                        None => return Err(BuildIssue::Parse(ParseIssue::ProblemSpecification)),
                        Some(string) => match string.parse() {
                            Err(_) => {
                                return Err(BuildIssue::Parse(ParseIssue::ProblemSpecification))
                            }
                            Ok(count) => count,
                        },
                    };

                    buffer.clear();

                    if config.show_stats {
                        println!("c Parsing {:#?}", file_path);
                        println!(
                            "c Expectation is to get {} variables and {} clauses",
                            variable_count, clause_count
                        );
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
                Err(_) => return Err(BuildIssue::Parse(ParseIssue::Line(line_counter))),
            }

            match buffer.chars().next() {
                Some('%') => break 'formula_loop,
                Some('c') => {}
                Some('p') => {
                    return Err(BuildIssue::Parse(ParseIssue::MisplacedProblem(
                        line_counter,
                    )))
                }
                _ => {
                    let split_buf = buffer.split_whitespace();
                    for item in split_buf {
                        match item {
                            "0" => {
                                let the_clause = clause_buffer.clone();
                                match the_context.preprocess_and_store_clause(the_clause) {
                                    Ok(_) => clause_counter += 1,
                                    Err(e) => return Err(e),
                                }

                                clause_buffer.clear();
                            }
                            _ => {
                                let the_literal = match the_context.literal_from_string(item) {
                                    Ok(literal) => literal,
                                    Err(e) => return Err(BuildIssue::Parse(e)),
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

        if show_stats {
            println!(
                "c Parsing complete with {} variables and {} clauses ({} added to the context)",
                the_context.variable_count(),
                clause_counter,
                the_context.clause_count()
            );
        }

        if the_context.clause_count() == 0 {
            return Err(BuildIssue::OopsAllTautologies);
        }

        Ok(the_context)
    }
}
