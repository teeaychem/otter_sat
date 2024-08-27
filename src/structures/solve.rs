use crate::structures::{ClauseId, Formula, Level, Literal, LiteralError, Valuation, ValuationVec};

use std::collections::BTreeSet;

#[derive(Debug)]
pub struct Solve {
    pub formula: &'static Formula,
    pub sat: Option<bool>,
    pub valuation: Vec<Option<bool>>,
    pub levels: Vec<Level>,
}

#[derive(Debug, PartialEq)]
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
    pub fn from_formula(formula: &'static Formula) -> Self {
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

    /* ideally the check on an ignored unit is improved
     for example, with watched literals a clause can be ignored in advance if the ignored literal is watched and it's negation is not part of the given valuation.
    whether this makes sense to do…
    */

    pub fn find_all_unset_on<T: Valuation>(
        &self,
        valuation: &T,
    ) -> (BTreeSet<(ClauseId, Literal)>, BTreeSet<Literal>) {
        let mut the_unit_set = BTreeSet::new();
        let mut the_choice_set = BTreeSet::new();
        for clause in self.formula.clauses.iter() {
            let the_unset = clause.collect_unset(valuation);
            if the_unset.len() == 1 {
                let the_pair: (ClauseId, Literal) = (clause.id, *the_unset.first().unwrap());
                the_unit_set.insert(the_pair);
                if the_choice_set.contains(&the_pair.1) {
                    the_choice_set.remove(&the_pair.1);
                }
            } else {
                for literal in the_unset {
                    the_choice_set.insert(literal);
                }
            }
        }
        (the_unit_set, the_choice_set)
    }

    pub fn literals_of_polarity(&self, polarity: bool) -> BTreeSet<Literal> {
        self.formula
            .clauses
            .iter()
            .fold(BTreeSet::new(), |mut acc: BTreeSet<Literal>, this| {
                acc.extend(
                    this.literals
                        .iter()
                        .filter(|&l| l.polarity == polarity)
                        .cloned()
                        .collect::<BTreeSet<Literal>>(),
                );
                acc
            })
    }

    // pub fn find_all_immediate_units_on<T: Valuation>(
    //     &self,
    //     valuation: &T,
    //     ignoring: &BTreeSet<(ClauseId, Literal)>,
    // ) -> Vec<(ClauseId, Literal)> {
    //     let mut the_set = BTreeSet::new();
    //     for clause in self.formula.clauses.iter() {
    //         if let Some(unit_literal) = clause.find_unit_literal(valuation) {
    //             let the_pair = (clause.id, unit_literal);
    //             if !ignoring.contains(&the_pair) {
    //                 the_set.insert(the_pair);
    //             }
    //         }
    //     }
    //     the_set.iter().cloned().collect()
    // }

    // pub fn find_all_units_on<T: Valuation + Clone>(
    //     &self,
    //     valuation: &T,
    //     ignoring: &mut BTreeSet<(ClauseId, Literal)>,
    // ) -> Vec<(ClauseId, Literal)> {
    //     let immediate_units = self.find_all_immediate_units_on(valuation, ignoring);
    //     ignoring.extend(immediate_units.clone());
    //     let mut further_units = vec![];
    //     if !immediate_units.is_empty() {
    //         for (_, literal) in &immediate_units {
    //             let mut updated_valuation = valuation.clone();
    //             let _ = updated_valuation.set_literal(literal);
    //             further_units.extend(
    //                 self.find_all_units_on(&updated_valuation, ignoring)
    //                     .iter()
    //                     .cloned(),
    //             );
    //         }
    //     }
    //     further_units.extend(immediate_units.iter().cloned());
    //     further_units
    // }
}

impl std::fmt::Display for Solve {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let _ = writeln!(f, "Valuation: {}", self.valuation.as_display_string(self));
        let _ = write!(f, "{}", self.formula);

        Ok(())
    }
}
