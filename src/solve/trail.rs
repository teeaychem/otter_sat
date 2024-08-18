use crate::structures::{
    Assignment, ClauseId, Literal, LiteralSource, Solve, SolveError, VariableId,
};

#[derive(Clone, Debug)]
pub struct Record {
    level: usize,
    literal: Literal,
    source: LiteralSource,
}

/// a partial assignment with some history
#[derive(Debug)]
pub struct Trail {
    level: usize,
    assignment: Assignment,
    records: Vec<Record>,
}

impl Trail {
    pub fn for_solve(solve: &Solve) -> Self {
        Trail {
            level: 0,
            assignment: Assignment::new(solve.vars().len()),
            records: vec![],
        }
    }

    // the last choice corresponds to the curent level
    pub fn undo_last_choice(&mut self) -> Option<Literal> {
        if let Some(record) = self.find_last_choice() {
            self.fresh_level(self.level);
            Some(record.literal)
        } else {
            None
        }
    }

    pub fn find_last_choice(&self) -> Option<Record> {
        self.records
            .iter()
            .find(|record| record.level == self.level && record.source == LiteralSource::Choice)
            .cloned()
    }

    pub fn level_start(&self, level: usize) -> Option<usize> {
        self.records.iter().position(|record| record.level == level)
    }

    /// creates a fresh decision level /l/ by clearing the records from levels ≥ /l/
    pub fn fresh_level(&mut self, l: usize) {
        self.records.iter().for_each(|record| {
            if record.level >= l {
                self.assignment.clear(record.literal.v_id());
            }
        });
        if let Some(position) = self.level_start(l) {
            self.records.truncate(position);
        }
        self.level = l;
    }

    pub fn set(&mut self, literal: &Literal, source: LiteralSource) {
        self.assignment.set(literal.clone());
        let record = match source {
            LiteralSource::Choice => {
                self.level += 1;
                Record {
                    level: self.level,
                    literal: literal.clone(),
                    source,
                }
            }
            LiteralSource::FreeChoice | LiteralSource::Assumption => Record {
                level: 0,
                literal: literal.clone(),
                source,
            },
            LiteralSource::Deduction => Record {
                level: self.level,
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
    pub fn trail_solve(&mut self) -> Result<(bool, Assignment), SolveError> {
        let mut the_trail = Trail::for_solve(self);
        let sat_assignment: Option<(bool, Assignment)>;

        loop {
            // 1. (un)sat check
            if self.is_sat_on(&the_trail.assignment) {
                sat_assignment = Some((true, the_trail.assignment.clone()));
                break;
            } else if self.is_unsat_on(&the_trail.assignment) {
                if let Some(literal) = the_trail.undo_last_choice() {
                    the_trail.set(&literal.negate(), LiteralSource::Deduction)
                } else {
                    sat_assignment = Some((false, the_trail.assignment.clone()));
                    break;
                }
            }
            // 2. search
            while let Some((lit, _clause)) = self.find_unit_on(&the_trail.assignment) {
                the_trail.set(&lit, LiteralSource::Deduction);
            }
            if let Some(v_id) = the_trail.get_unassigned_id(self) {
                if self.clauses.iter().any(|clause| {
                    clause
                        .literals()
                        .iter()
                        .filter(|l| l.v_id() == v_id)
                        .count()
                        > 0
                }) {
                    the_trail.set(&Literal::new(v_id, true), LiteralSource::Choice);
                } else {
                    the_trail.set(&Literal::new(v_id, true), LiteralSource::FreeChoice);
                }
            }
        }
        match sat_assignment {
            Some(pair) => Ok(pair),
            None => Err(SolveError::Hek),
        }
    }
}
