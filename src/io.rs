use std::char;

use crate::structures::{Formula, SolveError};

pub enum IOError {
    ParseFailure,
    PrefaceLength,
    PrefaceFormat,
    AddClauseFailure(SolveError),
}

impl Formula {
    pub fn from_dimacs(string: &str) -> Result<Formula, IOError> {
        let mut the_solve = Formula::new();
        let mut from = 0;
        let mut to = 0;
        let mut reading_comment = false;
        let mut reading_literal = false;
        while let Some(ch) = string.chars().nth(to) {
            if !reading_literal {
                if ['-', '1', '2', '3', '4', '5', '6', '7', '8', '9'].contains(&ch) {
                    reading_literal = true
                } else if ch == '0' {
                    if !reading_comment {
                        the_solve.add_clause(&string[from..to])?;
                    }
                    from = to + 1;
                }
            }
            if reading_literal && ch.is_whitespace() {
                reading_literal = false
            }

            if ch == 'c' {
                reading_comment = true;
            } else if ch == 0xA as char {
                // newline check
                from += 1;
                reading_comment = false;
            } else if !reading_comment && ch == 'p' {
                loop {
                    to += 1;
                    if let Some(other_ch) = string.chars().nth(to) {
                        if other_ch == 0xA as char {
                            break;
                        }
                    } else {
                        return Err(IOError::ParseFailure);
                    }
                }
                let the_preface = &string[from..to];
                let preface_parts = the_preface.split_whitespace().collect::<Vec<_>>();
                if preface_parts.len() != 4 {
                    return Err(IOError::PrefaceLength);
                } else if Some(&"p") != preface_parts.first()
                    || Some(&"cnf") != preface_parts.get(1)
                {
                    return Err(IOError::PrefaceFormat);
                }
                let _variables = match preface_parts.get(2) {
                    Some(count) => match count.parse::<usize>() {
                        Ok(count_number) => count_number,
                        Err(_) => return Err(IOError::ParseFailure),
                    },
                    None => return Err(IOError::ParseFailure),
                };
                let _clauses = match preface_parts.get(3) {
                    Some(count) => match count.parse::<usize>() {
                        Ok(count_number) => count_number,
                        Err(_) => return Err(IOError::ParseFailure),
                    },
                    None => return Err(IOError::ParseFailure),
                };
                from = to
            }

            if reading_comment {
                from += 1
            }
            to += 1;
        }
        Ok(the_solve)
    }

    pub fn add_clause(&mut self, string: &str) -> Result<(), IOError> {
        let string_lterals = string.split_whitespace();
        let mut the_clause = vec![];
        for string_literal in string_lterals {
            match self.literal_from_string(string_literal) {
                Ok(made) => the_clause.push(made),
                Err(e) => {
                    return Err(IOError::AddClauseFailure(e));
                }
            };
        }
        self.store_clause(the_clause);
        Ok(())
    }
}
