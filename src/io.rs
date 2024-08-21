use crate::structures::*;

enum IOError {
    UnexpectedInformation,
}

impl Solve {
    // todo, make this an iterator?
    pub fn from_dimacs(string: &str) -> Result<Solve, SolveError> {
        let mut the_solve = Solve::new();

        let mut from = 0;
        let mut to = 0;
        while let Some(ch) = string.chars().nth(to) {
            if ch == '0' {
                the_solve.add_clause(&string[from..to])?;
                from = to + 1;
            }
            to += 1;
        }

        if !&string[from..].trim().is_empty() {
            Err(SolveError::ParseFailure)
        } else {
            Ok(the_solve)
        }
    }
}

impl Solve {
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
