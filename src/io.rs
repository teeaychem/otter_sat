use core::char;
use std::fmt::Display;
use std::io::{stdout, Write};

use crate::{structures::formula::Formula, Config};

impl Formula {
    pub fn from_dimacs(string: &str) -> Self {
        let mut the_formula = Self::new();
        let mut from = 0;
        let mut to = 0;
        let mut reading_comment = false;
        let mut reading_literal = false;
        while let Some(ch) = string.chars().nth(to) {
            if !reading_literal {
                if ['-', '1', '2', '3', '4', '5', '6', '7', '8', '9'].contains(&ch) {
                    reading_literal = true;
                } else if ch == '0' {
                    if !reading_comment {
                        the_formula.add_clause(&string[from..to]);
                    }
                    from = to + 1;
                }
            }
            if reading_literal && ch.is_whitespace() {
                reading_literal = false;
            }

            if ch == 'c' {
                reading_comment = true;
                from += 1;
            } else if ch == 0xA as char {
                // newline check
                from = to;
                reading_comment = false;
            } else if !reading_comment && ch == 'p' {
                loop {
                    to += 1;
                    if string.chars().nth(to).expect("IO: Parse failure") == 0xA as char {
                        break;
                    }
                }
                let the_preface = &string[from..to];
                let preface_parts = the_preface.split_whitespace().collect::<Vec<_>>();

                assert!(preface_parts.len() == 4, "IO: Puzzled by preface length");
                assert!(preface_parts[0] == "p", "IO: Puzzled by preface format");
                assert!(preface_parts[1] == "cnf", "IO: Puzzled by preface format");

                let _variables = match preface_parts[2].parse::<usize>() {
                    Ok(count_number) => count_number,
                    Err(e) => panic!("IO: Parse failure {e:?}"),
                };

                let _clauses = match preface_parts[3].parse::<usize>() {
                    Ok(count_number) => count_number,
                    Err(e) => panic!("IO: Parse failure {e:?}"),
                };
                from = to;
            }

            to += 1;
        }
        the_formula
    }
}

use crossterm::{cursor, terminal, QueueableCommand};

pub struct ContextWindow {
    location: (u16, u16),
    column: u16,
    top: u16,
    time_limit: bool,
}

#[derive(Debug, Clone, Copy)]
pub enum WindowItem {
    Iterations,
    Conflicts,
    Ratio,
    Time,
}

impl ContextWindow {
    pub fn new(location: (u16, u16), config: &Config, formula: &Formula) -> Self {
        println!("c 🦦");
        println!("c Parsing formula from file: {:?}", config.formula_file);
        println!(
            "c Parsed formula with {} variables and {} clauses",
            formula.variable_count(),
            formula.clause_count()
        );
        println!("c CHOICE POLARITY LEAN {}", config.polarity_lean);
        if let Some(limit) = config.time_limit {
            println!("c TIME LIMIT: {:.2?}", limit);
        }
        println!("c ITERATIONS");
        println!("c CONFLCITS");
        println!("c RATIO");
        println!("c TIME");
        ContextWindow {
            location,
            column: 14,
            top: location.1,
            time_limit: config.time_limit.is_some(),
        }
    }
    fn get_offset(&self, item: WindowItem) -> (u16, u16) {
        let mut the_row = self.top;
        if self.time_limit {
            the_row += 1
        }
        match item {
            WindowItem::Iterations => the_row += 4,
            WindowItem::Conflicts => the_row += 5,
            WindowItem::Ratio => the_row += 6,
            WindowItem::Time => the_row += 7,
        }
        (self.column, the_row)
    }

    pub fn update_item(&self, item: WindowItem, i: impl Display) {
        let mut stdout = stdout();
        stdout.queue(cursor::SavePosition).unwrap();
        let (x, y) = self.get_offset(item);
        let _ = stdout.queue(cursor::MoveTo(x, y));
        stdout
            .queue(terminal::Clear(terminal::ClearType::UntilNewLine))
            .unwrap();
        match item {
            WindowItem::Ratio => stdout.write_all(format!("{i:.4}").as_bytes()).unwrap(),
            _ => stdout.write_all(format!("{i}").as_bytes()).unwrap(),
        }
        stdout.queue(cursor::RestorePosition).unwrap();
    }

    pub fn flush(&self) {
        stdout().flush().unwrap();
    }
}
