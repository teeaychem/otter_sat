use crate::{
    context::{config::Config, Context},
    structures::{
        clause::stored::Source as ClauseSource,
        literal::{Literal, Source as LiteralSource},
        variable::{list::VariableList, Status as VariableStatus, Variable, VariableId},
    },
};

use std::{
    fs::File,
    io::{BufRead, BufReader},
    path::Path,
};

#[derive(Debug)]
pub enum BuildIssue {
    UnitClauseConflict,
    AssumptionConflict,
}

impl Context {
    pub fn literal_from_string(&mut self, string: &str) -> Literal {
        let trimmed_string = string.trim();
        assert!(
            !trimmed_string.is_empty() && trimmed_string != "-",
            "No variable when creating literal from string"
        );
        let polarity = !trimmed_string.starts_with('-');

        let mut the_name = trimmed_string;
        if !polarity {
            the_name = &the_name[1..];
        }

        self.literal_ensure(the_name, polarity)
    }

    pub fn literal_ensure(&mut self, name: &str, polarity: bool) -> Literal {
        let the_variable = {
            match self.variables.string_map.get(name) {
                Some(variable) => *variable,
                None => {
                    let the_id = self.variables.len() as VariableId;
                    self.variables.add_variable(Variable::new(name, the_id));
                    the_id
                }
            }
        };
        Literal::new(the_variable, polarity)
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
            let the_literal = self.literal_from_string(string_literal);
            the_clause.push(the_literal);
        }
        the_clause.sort_unstable();
        the_clause.dedup();

        assert!(
            !the_clause.is_empty(),
            "c The formula contains an empty clause"
        );

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
                self.store_clause(the_clause, ClauseSource::Formula);
                Ok(())
            }
        }
    }

    #[allow(clippy::manual_flatten)]
    pub fn from_dimacs(file: &Path, config: &Config) -> Self {
        let file = File::open(file).unwrap();

        let mut buffer = String::with_capacity(1024);
        let mut file_reader = BufReader::new(file);
        let mut clause_buffer: Vec<Literal> = Vec::new();

        let mut the_context = None;

        // first phase, read until the formula begins
        loop {
            match file_reader.read_line(&mut buffer) {
                Ok(0) => break,
                Ok(_) => {}
                Err(e) => panic!("error reading line {e:?}"),
            }

            match buffer.chars().next() {
                Some('c') => {
                    buffer.clear();
                    continue;
                }
                Some('p') => {
                    let mut problem_details = buffer.split_whitespace();
                    let variable_count: usize = problem_details
                        .nth(2)
                        .expect("bad problem spec variable")
                        .parse()
                        .expect("bad variable parse");

                    let clause_count: usize = problem_details
                        .next()
                        .expect("bad problem spec clause")
                        .parse()
                        .expect("bad clause parse");

                    buffer.clear();

                    if config.show_stats {
                        println!(
                            "c Parsing {}",
                            config.formula_file.clone().unwrap().display()
                        );
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

        // second phase, read the formula to the context
        loop {
            match file_reader.read_line(&mut buffer) {
                Ok(0) => break,
                Ok(_) => {}
                Err(e) => panic!("error reading line {e:?}"),
            }

            match buffer.chars().next() {
                Some('c') => {}
                Some('p') => panic!("problem specified within formula details"),
                _ => {
                    let split_buf = buffer.split_whitespace();
                    for item in split_buf {
                        match item {
                            "0" => {
                                let mut the_clause = clause_buffer.clone();
                                the_clause.sort_unstable();
                                the_clause.dedup();
                                the_context.store_clause(the_clause, ClauseSource::Formula);
                                clause_buffer.clear();
                            }
                            _ => {
                                let literal = the_context.literal_from_string(item);
                                clause_buffer.push(literal);
                            }
                        }
                    }
                }
            }

            buffer.clear();
        }

        if config.show_stats {
            println!(
                "c Parsing complete with {} variables and {} clauses",
                the_context.variables().slice().len(),
                the_context.clause_count()
            );
        }

        the_context
    }
}
