use crate::structures::{
    Clause, ClauseId, ClauseVec, Literal, LiteralError, SolveError, StoredClause, Variable,
    VariableId,
};

use std::sync::atomic::{AtomicUsize, Ordering as AtomicOrdering};

#[derive(Debug, Clone)]
pub struct Formula {
    variables: Vec<Variable>,
    clauses: Vec<ClauseVec>,
}

impl Formula {
    pub fn new() -> Self {
        Formula {
            variables: vec![],
            clauses: vec![],
        }
    }

    fn fresh_clause_id() -> ClauseId {
        static COUNTER: AtomicUsize = AtomicUsize::new(1);
        COUNTER.fetch_add(1, AtomicOrdering::Relaxed) as ClauseId
    }

    // pub fn fresh_clause_from(&self, optional_clause: Option<impl Clause>) -> StoredClause {
    //     match optional_clause {
    //         Some(clause) => StoredClause {
    //             id: Self::fresh_clause_id(),
    //             clause: clause.to_vec(),
    //         },
    //         None => StoredClause {
    //             id: Self::fresh_clause_id(),
    //             clause: Vec::new(),
    //         },
    //     }
    // }

    pub fn clauses(&self) -> impl Iterator<Item = &impl Clause> {
        self.clauses.iter()
    }

    pub fn vars(&self) -> &Vec<Variable> {
        &self.variables
    }

    pub fn var_id_by_name(&mut self, name: &str) -> VariableId {
        if let Some(variable) = self.variables.iter().find(|v| v.name == name) {
            variable.id
        } else {
            let the_id = self.variables.len() as VariableId;
            let new_variable = Variable {
                name: name.to_string(),
                decision_level: None,
                id: the_id,
            };
            self.variables.push(new_variable);
            the_id
        }
    }

    pub fn literal_from_string(&mut self, string: &str) -> Result<Literal, SolveError> {
        let trimmed_string = string.trim();

        if trimmed_string.is_empty() || trimmed_string == "-" {
            return Err(SolveError::Literal(LiteralError::NoVariable));
        }

        let polarity = trimmed_string.chars().nth(0) != Some('-');

        let mut the_name = trimmed_string;
        if !polarity {
            the_name = &the_name[1..]
        }

        let the_variable = self.var_id_by_name(the_name);
        let the_literal = Literal::new(the_variable, polarity);
        Ok(the_literal)
    }

    pub fn var_count(&self) -> usize {
        self.variables.len()
    }

    fn clause_vec_from_string(&mut self, string: &str) -> Result<Vec<Literal>, SolveError> {
        let string_lterals = string.split_whitespace();
        let mut the_clause = vec![];
        for string_literal in string_lterals {
            match self.literal_from_string(string_literal) {
                Ok(made) => the_clause.push(made),
                Err(e) => {
                    return Err(e);
                }
            };
        }
        Ok(the_clause)
    }

    pub fn add_clause(&mut self, string: &str) -> Result<(), SolveError> {
        match self.clause_vec_from_string(string) {
            Ok(a_clause) => {
                self.clauses.push(a_clause);
                Ok(())
            }
            Err(e) => Err(e),
        }
    }
}

impl std::fmt::Display for Formula {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        writeln!(f, "| Variables")?;
        writeln!(
            f,
            "|   {}",
            self.variables
                .iter()
                .map(|v| v.name.clone())
                .collect::<Vec<_>>()
                .join(" ")
        )?;
        writeln!(f, "| Clauses")?;
        for clause in &self.clauses {
            writeln!(f, "|   {}", clause.as_string())?;
        }
        Ok(())
    }
}
