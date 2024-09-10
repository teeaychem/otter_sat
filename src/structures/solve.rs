use crate::structures::{
    Clause, ClauseId, Formula, ImplicationEdge, ImplicationGraph, ImplicationNode, Level, Literal,
    LiteralError, LiteralSource, Valuation, ValuationError, ValuationVec, VariableId,
};
use std::sync::atomic::{AtomicUsize, Ordering as AtomicOrdering};

use std::collections::BTreeSet;

#[derive(Debug)]
pub struct Solve<'formula> {
    pub formula: &'formula Formula,
    pub sat: Option<bool>,
    pub valuation: Vec<Option<bool>>,
    pub levels: Vec<Level>,
    pub clauses: Vec<Clause>,
    pub graph: ImplicationGraph,
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
    SetIssue,
}

impl Solve<'_> {
    pub fn from_formula(formula: &Formula) -> Solve {
        let valuation = Vec::<Option<bool>>::new_for_variables(formula.vars().len());
        let mut the_solve = Solve {
            formula,
            sat: None,
            valuation,
            levels: vec![],
            clauses: vec![],
            graph: ImplicationGraph::new_for(formula),
        };
        let level_zero = Level::new(0, &the_solve);
        the_solve.levels.push(level_zero);
        the_solve
    }
}

// SAT related things
impl Solve<'_> {
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
        for clause in &self.formula.clauses {
            if let Some(the_unset) = clause.collect_choices(valuation) {
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

    pub fn fresh_clause_id() -> ClauseId {
        static COUNTER: AtomicUsize = AtomicUsize::new(1);
        COUNTER.fetch_add(1, AtomicOrdering::Relaxed) as ClauseId
    }

    pub fn get_unassigned_id(&self, solve: &Solve) -> Option<VariableId> {
        solve
            .formula
            .vars()
            .iter()
            .find(|&v| self.valuation.of_v_id(v.id).is_ok_and(|p| p.is_none()))
            .map(|found| found.id)
    }
}

impl<'borrow, 'solve> Solve<'solve> {
    pub fn learn_as_clause(&'borrow mut self, literals: Vec<Literal>) {
        panic!("learn as clause");
        let clause = Clause {
            id: Solve::fresh_clause_id(),
            position: self.clauses.len(),
            literals,
        };
        self.clauses.push(clause)
    }

    pub fn set_literal(
        &'borrow mut self,
        literal: &Literal,
        source: LiteralSource,
    ) -> Result<(), ValuationError> {
        match source {
            LiteralSource::Choice => {
                self.add_fresh_level();
                let current_level = self.current_level_index();
                self.levels[current_level].choices.push(*literal);
            }
            LiteralSource::HobsonChoice | LiteralSource::Assumption => {
                self.levels[0].observations.push(*literal);
            }
            LiteralSource::Clause(_) | LiteralSource::Conflict => {
                let current_level = self.current_level_index();
                self.levels[current_level].observations.push(*literal);
            }
        };
        let result = self.valuation.set_literal(literal);
        if Some(false) != self.sat {
            let current_level = self.current_level_index();
            if let Err(ValuationError::Inconsistent) = result {
                match source {
                    LiteralSource::Clause(c) => self.levels[current_level].clauses_violated.push(c),
                    _ => panic!("unsat without a clause"),
                }
                self.sat = Some(false)
            }
        }
        result
    }
}

impl std::fmt::Display for Solve<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let _ = writeln!(f, "Valuation: {}", self.valuation.as_display_string(self));
        let _ = write!(f, "{}", self.formula);

        Ok(())
    }
}
