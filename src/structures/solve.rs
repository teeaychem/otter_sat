use crate::structures::{
    Clause, ClauseId, Formula, Level, Literal, LiteralError, Valuation, ValuationVec, Variable,
    VariableId,
};

use std::collections::BTreeSet;

#[derive(Debug)]
pub struct Solve {
    pub formula: Formula,
    pub sat: Option<bool>,
    pub valuation: Vec<Option<bool>>,
    pub levels: Vec<Level>,
}

#[derive(Debug ,PartialEq)]
pub enum SolveError {
    Literal(LiteralError),
    // Clause(ClauseError),
    ParseFailure,
    PrefaceLength,
    PrefaceFormat,
    Hek,
    OutOfBounds,
}

impl Solve {
    pub fn from_formula(formula: Formula) -> Self {
        let valuation = Vec::<Option<bool>>::new_for_variables(formula.vars().len());
        let mut the_solve = Solve {
            formula,
            sat: None,
            valuation,
            levels: vec![],
        };
        let level_zero = Level::new(0, &the_solve);
        the_solve.levels.push(level_zero);
        the_solve
    }
}

// SAT related things
impl Solve {
    pub fn is_unsat_on(&self, valuation: &ValuationVec) -> bool {
        self.formula
            .clauses
            .iter()
            .any(|clause| clause.is_unsat_on(valuation))
    }

    pub fn is_sat_on(&self, valuation: &ValuationVec) -> bool {
        self.formula
            .clauses
            .iter()
            .all(|clause| clause.is_sat_on(valuation))
    }

    pub fn find_unit_on(&self, valuation: &ValuationVec) -> Option<(ClauseId, Literal)> {
        for clause in self.formula.clauses.iter() {
            if let Some(unit_literal) = clause.find_unit_literal(valuation) {
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
        valuation: &T,
        ignoring: &BTreeSet<(ClauseId, Literal)>,
    ) -> BTreeSet<(ClauseId, Literal)> {
        let mut the_set = BTreeSet::new();
        for clause in self.formula.clauses.iter() {
            if let Some(unit_literal) = clause.find_unit_literal(valuation) {
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
        write!(f, "{:?}\n", self.valuation);
        write!(f, "{}", self.formula);

        Ok(())
    }
}
