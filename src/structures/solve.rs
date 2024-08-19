use crate::structures::{
    Valuation, Clause, ClauseError, ClauseId, Literal, LiteralError, Variable, VariableId,
};

use std::sync::atomic::{AtomicUsize, Ordering as AtomicOrdering};

#[derive(Debug)]
pub struct Solve {
    variables: Vec<Variable>,
    pub clauses: Vec<Clause>,
}

#[derive(Debug)]
pub enum SolveError {
    Literal(LiteralError),
    // Clause(ClauseError),
    ParseFailure,
    Hek,
}

impl Solve {
    pub fn new() -> Self {
        Solve {
            variables: Vec::new(),
            clauses: Vec::new(),
        }
    }
}

// Variables and literal things
impl Solve {
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
}

// Clause things
impl Solve {
    fn make_clause_id() -> ClauseId {
        static COUNTER: AtomicUsize = AtomicUsize::new(1);
        COUNTER.fetch_add(1, AtomicOrdering::Relaxed) as ClauseId
    }

    pub fn fresh_clause() -> Clause {
        Clause::new(Self::make_clause_id())
    }
}

// SAT related things
impl Solve {
    pub fn is_unsat_on(&self, assignment: &Valuation) -> bool {
        self.clauses
            .iter()
            .any(|clause| clause.is_unsat_on(assignment))
    }

    pub fn is_sat_on(&self, assignment: &Valuation) -> bool {
        self.clauses
            .iter()
            .all(|clause| clause.is_sat_on(assignment))
    }

    pub fn find_unit_on(&self, assignment: &Valuation) -> Option<(Literal, ClauseId)> {
        for clause in self.clauses.iter() {
            if let Some(unit_literal) = clause.find_unit_literal(assignment) {
                return Some((unit_literal, clause.id()));
            }
        }
        None
    }

    pub fn all_units_on(&self, assignment: &Valuation) -> Vec<(Literal, ClauseId)> {
        let mut the_vec = vec![];
        for clause in self.clauses.iter() {
            if let Some(unit_literal) = clause.find_unit_literal(assignment) {
                the_vec.push((unit_literal, clause.id()));
            }
        }
        the_vec
    }
}
