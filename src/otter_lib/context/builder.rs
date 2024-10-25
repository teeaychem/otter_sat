use crate::{
    config::Config,
    context::Context,
    structures::{
        clause::stored::Source as ClauseSource,
        literal::{Literal, Source as LiteralSource},
        variable::{list::VariableList, Variable, VariableId},
    },
};

use std::{
    fs::File,
    io::{BufRead, BufReader},
    path::PathBuf,
};

#[derive(Debug)]
pub enum BuildIssue {
    UnitClauseConflict,
    AssumptionConflict,
    ClauseEmpty,
    Parse(ParseIssue),
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
            match self.variables.string_map.get(the_name) {
                Some(variable) => *variable,
                None => {
                    let the_id = self.variables.len() as VariableId;
                    self.variables.add_variable(Variable::new(the_name, the_id));
                    the_id
                }
            }
        };
        Ok(Literal::new(the_variable, polarity))
    }

    pub fn assume_literal(&mut self, literal: Literal) -> Result<(), BuildIssue> {
        match self.variables.set_value(
            literal,
            unsafe { self.levels.get_unchecked_mut(0) },
            LiteralSource::Assumption,
        ) {
            Ok(_) => Ok(()),
            Err(_e) => Err(BuildIssue::AssumptionConflict),
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
            the_clause.push(the_literal);
        }
        the_clause.sort_unstable();
        the_clause.dedup();

        if the_clause.is_empty() {
            return Err(BuildIssue::ClauseEmpty);
        }

        match the_clause.len() {
            1 => {
                match self.variables.set_value(
                    *the_clause.first().expect("literal vanish"),
                    unsafe { self.levels.get_unchecked_mut(0) },
                    LiteralSource::Assumption,
                ) {
                    Ok(_) => Ok(()),
                    Err(_e) => Err(BuildIssue::UnitClauseConflict),
                }
            }
            _ => {
                // temp taut check
                let mut tautology = false;
                for literal in &the_clause {
                    if the_clause
                        .iter()
                        .find(|l| **l == literal.negate())
                        .is_some()
                    {
                        tautology = true;
                        break;
                    }
                }

                if !tautology {
                    self.store_clause(the_clause, ClauseSource::Formula);
                }
                Ok(())
            }
        }
    }

    #[allow(clippy::manual_flatten, unused_labels)]
    pub fn from_dimacs(file_path: &PathBuf, config: Config) -> Result<Self, BuildIssue> {
        let file = match File::open(file_path) {
            Err(_) => return Err(BuildIssue::Parse(ParseIssue::NoFile)),
            Ok(f) => f,
        };

        let mut buffer = String::with_capacity(1024);
        let mut file_reader = BufReader::new(file);
        let mut clause_buffer: Vec<Literal> = Vec::new();

        let mut the_context = None;
        let mut line_counter = 0;
        let mut assumption_counter = 0;

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
                        config,
                    ));
                    break;
                }
                _ => {
                    break;
                }
            }
        }

        let mut the_context = the_context.unwrap_or_default();

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
                                let mut the_clause = clause_buffer.clone();
                                the_clause.sort_unstable();
                                the_clause.dedup();

                                match the_clause.len() {
                                    1 => {
                                        assumption_counter += 1;
                                        match the_context.variables.set_value(
                                            *the_clause.first().expect("literal vanish"),
                                            unsafe { the_context.levels.get_unchecked_mut(0) },
                                            LiteralSource::Assumption,
                                        ) {
                                            Ok(_) => {}
                                            Err(_e) => return Err(BuildIssue::UnitClauseConflict),
                                        }
                                    }
                                    _ => {
                                        // temp taut check
                                        let mut tautology = false;
                                        for literal in &the_clause {
                                            if the_clause
                                                .iter()
                                                .find(|l| **l == literal.negate())
                                                .is_some()
                                            {
                                                tautology = true;
                                                break;
                                            }
                                        }

                                        if !tautology {
                                            the_context
                                                .store_clause(the_clause, ClauseSource::Formula);
                                        }
                                    }
                                }

                                clause_buffer.clear();
                            }
                            _ => {
                                let the_literal = match the_context.literal_from_string(item) {
                                    Ok(literal) => literal,
                                    Err(e) => return Err(BuildIssue::Parse(e)),
                                };
                                clause_buffer.push(the_literal);
                            }
                        }
                    }
                }
            }

            buffer.clear();
        }

        if show_stats {
            println!(
                "c Parsing complete with {} variables and {} clauses",
                the_context.variables().slice().len(),
                the_context.clause_count() + assumption_counter
            );
        }

        Ok(the_context)
    }
}
