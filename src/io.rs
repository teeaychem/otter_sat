use crate::structures::*;

enum IOError {
    UnexpectedInformation,
}

impl Solve {
    // todo, make this an iterator?
    pub fn from_dimacs(string: &str) -> Result<Solve, SolveError> {
        println!("solve from dimacs");
        let mut the_solve = Solve::new();

        let mut from = 0;
        let mut to = 0;
        while let Some(ch) = string.chars().nth(to) {
            if ch == '0' {
                println!("adding clause");
                the_solve.add_clause(&string[from..to])?;
                println!("added clause");
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
        let mut the_clause = Self::fresh_clause();
        for string_literal in string_lterals {
            println!("making {string_literal}");
            let _ = match self.literal_from_string(string_literal) {
                Ok(made) => the_clause.add_literal(made),
                Err(e) => {
                    println!("but error");
                    return Err(e);
                }
            };
        }
        self.clauses.push(the_clause);
        Ok(())
    }
}

// impl Clause {
//     pub fn from_dimacs(string: &str) -> Result<Clause, ClauseError> {}
// }

// impl Cnf {
//     pub fn from_dimacs(string: &str) -> Result<Cnf, CnfError> {
//         let mut cnf = Cnf::new();

//         let mut from = 0;
//         let mut to = 0;
//         while let Some(ch) = string.chars().nth(to) {
//             if ch == '0' {
//                 match Clause::from_dimacs(&string[from..to]) {
//                     Ok(clause) => cnf.add_clause(clause),
//                     Err(e) => {
//                         return Err(CnfError::Clause(e));
//                     }
//                 };
//                 from = to + 1;
//             }
//             to += 1;
//         }

//         if !&string[from..].trim().is_empty() {
//             Err(CnfError::UnexpectedInformation)
//         } else {
//             Ok(cnf)
//         }
//     }
// }
