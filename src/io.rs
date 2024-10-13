use core::char;

use crate::structures::formula::Formula;

impl Formula {
    pub fn from_dimacs(string: &str) -> Self {
        let mut the_solve = Self::new();
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
                        the_solve.add_clause(&string[from..to]);
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
        the_solve
    }
}
