use crate::structures::{
    Clause, ClauseId, Literal, LiteralError, Valuation, ValuationVec, Variable, VariableId,
};

use std::collections::BTreeSet;

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
    PrefaceLength,
    PrefaceFormat,
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
    // use std::sync::atomic::{AtomicUsize, Ordering as AtomicOrdering};
    // fn make_clause_id() -> ClauseId {
    //     static COUNTER: AtomicUsize = AtomicUsize::new(1);
    //     COUNTER.fetch_add(1, AtomicOrdering::Relaxed) as ClauseId
    // }

    pub fn fresh_clause(&self) -> Clause {
        Clause::new(self.clauses.len())
    }
}

// SAT related things
impl Solve {
    pub fn is_unsat_on(&self, assignment: &ValuationVec) -> bool {
        self.clauses
            .iter()
            .any(|clause| clause.is_unsat_on(assignment))
    }

    pub fn is_sat_on(&self, assignment: &ValuationVec) -> bool {
        self.clauses
            .iter()
            .all(|clause| clause.is_sat_on(assignment))
    }

    pub fn find_unit_on(&self, assignment: &ValuationVec) -> Option<(ClauseId, Literal)> {
        for clause in self.clauses.iter() {
            if let Some(unit_literal) = clause.find_unit_literal(assignment) {
                return Some((clause.id, unit_literal));
            }
        }
        None
    }

    /* ideally the check on an ignored unit is improved
     for example, with watched literals a clause can be ignored in advance if the ignored literal is watched and it's negation is not part of the given valuation.
    whether this makes sense to do…
    */

    pub fn all_immediate_units_on<T: Valuation>(
        &self,
        assignment: &T,
        ignoring: &BTreeSet<(ClauseId, Literal)>,
    ) -> BTreeSet<(ClauseId, Literal)> {
        let mut the_set = BTreeSet::new();
        for clause in self.clauses.iter() {
            if let Some(unit_literal) = clause.find_unit_literal(assignment) {
                let the_pair = (clause.id, unit_literal);
                if !ignoring.contains(&the_pair) {
                    the_set.insert(the_pair);
                }
            }
        }
        the_set
    }

    pub fn find_all_units_on<T: Valuation + Clone>(
        &self,
        valuation: &T,
        ignoring: &mut BTreeSet<(ClauseId, Literal)>,
    ) -> Vec<(ClauseId, Literal)> {
        let immediate_units = self.all_immediate_units_on(valuation, ignoring);
        ignoring.extend(immediate_units.clone());
        let mut further_units = vec![];
        if !immediate_units.is_empty() {
            for (_, literal) in &immediate_units {
                let mut updated_valuation = valuation.clone();
                updated_valuation.set_literal(literal);
                further_units.extend(
                    self.find_all_units_on(&updated_valuation, ignoring)
                        .iter()
                        .cloned(),
                );
            }
        }
        further_units.extend(immediate_units.iter().cloned());
        further_units
    }
}
