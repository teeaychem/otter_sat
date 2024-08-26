use std::char;

use crate::structures::*;

enum IOError {
    UnexpectedInformation,
}

impl Formula {
    pub fn from_dimacs(string: &str) -> Result<Formula, SolveError> {
        let mut the_solve = Formula::new();
        let mut from = 0;
        let mut to = 0;
        let mut skip = false;
        while let Some(ch) = string.chars().nth(to) {
            if ch == '0' {
                if !skip {
                    the_solve.add_clause(&string[from..to])?;
                }
                from = to + 1;
            } else if ch == 'c' {
                skip = true;
            } else if ch == 0xA as char {
                // newline check
                from += 1;
                skip = false;
            } else if ch == 'p' {
                loop {
                    to += 1;
                    if let Some(other_ch) = string.chars().nth(to) {
                        if other_ch == 0xA as char {
                            break;
                        }
                    } else {
                        return Err(SolveError::ParseFailure);
                    }
                }
                let the_preface = &string[from..to];
                let mut preface_parts = the_preface.split_whitespace();
                preface_parts.next(); // skip 'p'
                if to - from < 7 {
                    return Err(SolveError::PrefaceLength);
                } else if Some("cnf") != preface_parts.next() {
                    return Err(SolveError::PrefaceFormat);
                }
                let _variables = match preface_parts.next() {
                    Some(count) => match count.parse::<usize>() {
                        Ok(count_number) => count_number,
                        Err(_) => return Err(SolveError::ParseFailure),
                    },
                    None => return Err(SolveError::ParseFailure),
                };
                let _clauses = match preface_parts.next() {
                    Some(count) => match count.parse::<usize>() {
                        Ok(count_number) => count_number,
                        Err(_) => return Err(SolveError::ParseFailure),
                    },
                    None => return Err(SolveError::ParseFailure),
                };
                from = to
            }

            if skip {
                from += 1
            }
            to += 1;
        }

        if !&string[from..].trim().is_empty() {
            Err(SolveError::ParseFailure)
        } else {
            Ok(the_solve)
        }
    }

    pub fn add_clause(&mut self, string: &str) -> Result<(), SolveError> {
        let string_lterals = string.split_whitespace();
        let mut the_clause = self.fresh_clause();
        for string_literal in string_lterals {
            let _ = match self.literal_from_string(string_literal) {
                Ok(made) => the_clause.add_literal(made),
                Err(e) => {
                    return Err(e);
                }
            };
        }
        self.clauses.push(the_clause);
        Ok(())
    }
}
