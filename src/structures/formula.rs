use crate::structures::{
    solve::SolveError, Clause, ClauseVec, Literal, Variable, VariableId,
};

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

    pub fn clauses(&self) -> impl Iterator<Item = &impl Clause> {
        self.clauses.iter()
    }

    pub fn add_clause(&mut self, string: &str) -> Result<(), SolveError> {
        match self.clause_vec_from_string(string) {
            Ok(a_clause) => {
                self.clauses.push(a_clause);
                Ok(())
            }
            Err(e) => {
                panic!("{e:?}");
            },
        }
    }

    pub fn vars(&self) -> &[Variable] {
        &self.variables
    }

    pub fn var_id_by_name(&mut self, name: &str) -> VariableId {
        if let Some(variable) = self.variables.iter().find(|v| v.name() == name) {
            variable.id()
        } else {
            let the_id = self.variables.len() as VariableId;
            let new_variable = Variable::new(name, the_id);
            self.variables.push(new_variable);
            the_id
        }
    }

    fn clause_vec_from_string(&mut self, string: &str) -> Result<Vec<Literal>, SolveError> {
        let string_lterals = string.split_whitespace();
        let mut the_clause = vec![];
        for string_literal in string_lterals {
            match Literal::from_string(string_literal, &mut self.variables) {
                Ok(made) => the_clause.push(made),
                Err(e) => {
                    panic!("{e:?}");
                }
            };
        }
        the_clause.sort_unstable();
        the_clause.dedup();
        Ok(the_clause)
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
                .map(|v| v.name())
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
