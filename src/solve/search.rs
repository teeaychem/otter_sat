use crate::structures::{
    Assignment, ClauseId, Literal, LiteralSource, Solve, SolveError, VariableId,
};
use std::collections::BTreeSet;

#[derive(Clone, Debug)]
pub struct Record {
    depth: usize,
    literal: Literal,
    source: LiteralSource,
}

/// a partial assignment with some history
#[derive(Debug)]
pub struct Search {
    depth: usize,
    assignment: Assignment,
    records: Vec<Record>,
}

impl Search {
    pub fn for_solve(solve: &Solve) -> Self {
        Search {
            depth: 0,
            assignment: Assignment::new(solve.vars().len()),
            records: vec![],
        }
    }

    // the last choice corresponds to the curent depth
    pub fn undo_last_choice(&mut self) -> Option<Literal> {
        if let Some(record) = self.find_last_choice() {
            self.raise_to_depth(self.depth);
            Some(record.literal)
        } else {
            None
        }
    }

    pub fn find_last_choice(&self) -> Option<Record> {
        self.records
            .iter()
            .find(|record| record.depth == self.depth && record.source == LiteralSource::Choice)
            .cloned()
    }

    pub fn depth_start(&self, depth: usize) -> Option<usize> {
        self.records.iter().position(|record| record.depth == depth)
    }

    /// creates a fresh decision depth /d/ by clearing the records from depths ≥ /d/
    pub fn raise_to_depth(&mut self, l: usize) {
        self.records.iter().for_each(|record| {
            if record.depth >= l {
                self.assignment.clear(record.literal.v_id());
            }
        });
        if let Some(position) = self.depth_start(l) {
            self.records.truncate(position);
        }
        self.depth = l;
    }

    pub fn set(&mut self, literal: &Literal, source: LiteralSource) {
        self.assignment.set(literal.clone());
        let record = match source {
            LiteralSource::Choice => {
                self.depth += 1;
                Record {
                    depth: self.depth,
                    literal: literal.clone(),
                    source,
                }
            }
            LiteralSource::FreeChoice | LiteralSource::Assumption => Record {
                depth: 0,
                literal: literal.clone(),
                source,
            },
            LiteralSource::DeductionClause(_) | LiteralSource::DeductionFalsum => Record {
                depth: self.depth,
                literal: literal.clone(),
                source,
            },
        };
        self.records.push(record);
    }

    pub fn get_unassigned_id(&self, solve: &Solve) -> Option<VariableId> {
        solve
            .vars()
            .iter()
            .find(|&v| {
                self.assignment
                    .get_by_variable(v)
                    .is_ok_and(|p| p.is_none())
            })
            .map(|found| found.id)
    }
}

impl Solve {
    pub fn single_deduction_solve(&mut self) -> Result<(bool, Assignment), SolveError> {
        let mut the_search = Search::for_solve(self);
        let sat_assignment: Option<(bool, Assignment)>;

        loop {
            // 1. (un)sat check
            if self.is_sat_on(&the_search.assignment) {
                sat_assignment = Some((true, the_search.assignment.clone()));
                break;
            } else if self.is_unsat_on(&the_search.assignment) {
                if let Some(literal) = the_search.undo_last_choice() {
                    the_search.set(&literal.negate(), LiteralSource::DeductionFalsum)
                } else {
                    sat_assignment = Some((false, the_search.assignment.clone()));
                    break;
                }
            }
            // 2. search
            while let Some((lit, clause_id)) = self.find_unit_on(&the_search.assignment) {
                the_search.set(&lit, LiteralSource::DeductionClause(clause_id));
            }
            if let Some(v_id) = the_search.get_unassigned_id(self) {
                if self.clauses.iter().any(|clause| {
                    clause
                        .literals()
                        .iter()
                        .filter(|l| l.v_id() == v_id)
                        .count()
                        > 0
                }) {
                    the_search.set(&Literal::new(v_id, true), LiteralSource::Choice);
                } else {
                    the_search.set(&Literal::new(v_id, true), LiteralSource::FreeChoice);
                }
            }
        }
        match sat_assignment {
            Some(pair) => Ok(pair),
            None => Err(SolveError::Hek),
        }
    }

    pub fn literals_of_polarity(&self, polarity: bool) -> BTreeSet<Literal> {
        self.clauses
            .iter()
            .fold(BTreeSet::new(), |mut acc: BTreeSet<Literal>, this| {
                acc.extend(
                    this.literals()
                        .iter()
                        .filter(|&l| l.polarity() == polarity)
                        .cloned()
                        .collect::<BTreeSet<Literal>>(),
                );
                acc
            })
    }

    /// general order for pairs related to booleans is 0 is false, 1 is true
    pub fn free_choices(&self) -> (Vec<VariableId>, Vec<VariableId>) {
        // let all_v_ids: BTreeSet<VariableId> = self.vars().iter().map(|v| v.id).collect();
        let the_true: BTreeSet<VariableId> = self
            .literals_of_polarity(true)
            .iter()
            .map(|l| l.v_id())
            .collect();
        let the_false: BTreeSet<VariableId> = self
            .literals_of_polarity(false)
            .iter()
            .map(|l| l.v_id())
            .collect();
        let the_intersection = the_true.intersection(&the_false).cloned().collect();
        let free_false = the_false.difference(&the_intersection).cloned().collect();
        let free_true = the_true.difference(&the_intersection).cloned().collect();
        (free_false, free_true)
    }

    pub fn settle_free_choices(&self, search: &mut Search) {
        let the_free_choices = self.free_choices();
        the_free_choices.0.iter().for_each(|&v_id| {
            let the_choice = Literal::new(v_id, false);
            search.set(&the_choice, LiteralSource::FreeChoice);
        });
        the_free_choices.1.iter().for_each(|&v_id| {
            let the_choice = Literal::new(v_id, true);
            search.set(&the_choice, LiteralSource::FreeChoice);
        });
    }

    pub fn alt_deduction_solve(&mut self) -> Result<(bool, Assignment), SolveError> {
        let mut the_search = Search::for_solve(self);
        let sat_assignment: Option<(bool, Assignment)>;
        // settle any free choices
        self.settle_free_choices(&mut the_search);

        loop {
            // 1. (un)sat check
            if self.is_sat_on(&the_search.assignment) {
                sat_assignment = Some((true, the_search.assignment.clone()));
                break;
            } else if self.is_unsat_on(&the_search.assignment) {
                if let Some(literal) = the_search.undo_last_choice() {
                    the_search.set(&literal.negate(), LiteralSource::DeductionFalsum)
                } else {
                    sat_assignment = Some((false, the_search.assignment.clone()));
                    break;
                }
            }
            // 2. search
            while let Some((lit, clause_id)) = self.find_unit_on(&the_search.assignment) {
                the_search.set(&lit, LiteralSource::DeductionClause(clause_id));
            }
            if let Some(v_id) = the_search.get_unassigned_id(self) {
                the_search.set(&Literal::new(v_id, true), LiteralSource::Choice);
            }
        }
        match sat_assignment {
            Some(pair) => Ok(pair),
            None => Err(SolveError::Hek),
        }
    }
}
