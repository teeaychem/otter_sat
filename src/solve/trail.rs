use crate::structures::{
    Assignment, ClauseId, Literal, LiteralSource, Solve, SolveError, VariableId,
};

/// a partial assignment with some history
// the assignment
#[derive(Debug)]
pub struct Trail {
    assignment: Assignment,
    history: Vec<(Literal, LiteralSource)>,
}

impl Trail {
    pub fn for_solve(solve: &Solve) -> Self {
        Trail {
            assignment: Assignment::new(solve.vars().len()),
            history: vec![],
        }
    }

    /// work back through steps of the trail, discarding the trail, and relaxing the assignment
    // here, some deduced literals may still hold, but for now the trail does not record multiple paths to a deduction
    pub fn track_back(&mut self, steps: usize) {
        (0..steps).for_each(|_| {
            if let Some((literal, _)) = self.history.pop() {
                self.assignment.clear(literal.v_id())
            }
        })
    }

    pub fn backsteps_to_choice(&self) -> Option<usize> {
        let mut back_point = self.history.len() - 1;
        let mut back_steps = 0;
        loop {
            if let Some((_, source)) = self.history.get(back_point) {
                match source {
                    LiteralSource::Choice => return Some(back_steps),
                    _ => {
                        if back_point != 0 {
                            back_point -= 1;
                            back_steps += 1;
                        } else {
                            return None;
                        }
                    }
                }
            }
        }
    }

    pub fn undo_choice(&mut self) -> Option<Literal> {
        let steps_to_take = self.backsteps_to_choice()?;
        self.track_back(steps_to_take);
        if let Some((literal, _)) = self.history.pop() {
            self.assignment.clear(literal.v_id());
            Some(literal)
        } else {
            None
        }
    }

    pub fn set(&mut self, literal: &Literal, source: LiteralSource) {
        self.history.push((literal.clone(), source));
        self.assignment.set(literal.clone());
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

    pub fn find_unit(&self, solve: &Solve) -> Option<(Literal, ClauseId)> {
        for clause in &solve.clauses {
            if let Some(pair) = clause.find_unit_on(&self.assignment) {
                return Some(pair);
            }
        }
        None
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
                if let Some(literal) = the_trail.undo_choice() {
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
