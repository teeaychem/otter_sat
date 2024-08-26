use crate::structures::{
    Clause, ClauseId, Literal, LiteralError, Valuation, ValuationVec, Variable, VariableId, Formula
};


use std::collections::BTreeSet;


#[derive(Debug)]
pub struct Solve {
    pub formula: Formula,
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
    pub fn from_formula(formula: Formula) -> Self {
        Solve {
            formula,
        }
    }
}

// SAT related things
impl Solve {
    pub fn is_unsat_on(&self, assignment: &ValuationVec) -> bool {
        self.formula
            .clauses
            .iter()
            .any(|clause| clause.is_unsat_on(assignment))
    }

    pub fn is_sat_on(&self, assignment: &ValuationVec) -> bool {
        self.formula
            .clauses
            .iter()
            .all(|clause| clause.is_sat_on(assignment))
    }

    pub fn find_unit_on(&self, assignment: &ValuationVec) -> Option<(ClauseId, Literal)> {
        for clause in self.formula.clauses.iter() {
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
        for clause in self.formula.clauses.iter() {
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
                let _ = updated_valuation.set_literal(literal);
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


impl std::fmt::Display for Solve {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.formula);
        Ok(())
    }
}
