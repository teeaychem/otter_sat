use crate::structures::*;

impl Clause {
    pub fn from_dimacs(string: &str) -> Result<Clause, ClauseError> {
        let literals = string.split_whitespace();
        let mut the_clause = Clause::new();
        for literal in literals {
            let _ = match Literal::from_string(literal) {
                Ok(made) => the_clause.add_literal(made),
                Err(e) => {
                    return Err(ClauseError::Literal(e));
                }
            };
        }
        Ok(the_clause)
    }
}

impl Cnf {
    pub fn from_dimacs(string: &str) -> Result<Cnf, CnfError> {
        let mut cnf = Cnf::new();

        let mut from = 0;
        let mut to = 0;
        while let Some(ch) = string.chars().nth(to) {
            if ch == '0' {
                match Clause::from_dimacs(&string[from..to]) {
                    Ok(clause) => cnf.add_clause(clause),
                    Err(e) => {
                        return Err(CnfError::Clause(e));
                    }
                };
                from = to + 1;
            }
            to += 1;
        }

        if !&string[from..].trim().is_empty() {
            Err(CnfError::UnexpectedInformation)
        } else {
            Ok(cnf)
        }
    }
}
