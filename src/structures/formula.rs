use crate::structures::{
    Clause, ClauseId, Literal, LiteralError, SolveError, Variable, VariableId,
};

use std::sync::atomic::{AtomicUsize, Ordering as AtomicOrdering};

#[derive(Debug, Clone)]
pub struct Formula {
    pub variables: Vec<Variable>,
    pub clauses: Vec<Clause>,
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

    pub fn fresh_clause(&self) -> Clause {
        Clause {
            id: Self::fresh_clause_id(),
            position: self.clauses.len(),
            literals: Vec::new(),
        }
    }

    // todo think about the structure to allow dropping clauses, etc
    pub fn borrow_clause_by_id(&self, id: usize) -> &Clause {
        if let Some(clause) = self.clauses.iter().find(|c| c.id == id) {
            clause
        } else {
            panic!("Searching for a phantom clause");
        }
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
                id: the_id,
            };
            self.variables.push(new_variable);
            the_id
        }
    }

    pub fn var_by_id(&self, id: VariableId) -> Option<&Variable> {
        self.variables.get(id as usize)
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
            writeln!(f, "|   {}", clause)?;
        }
        Ok(())
    }
}
