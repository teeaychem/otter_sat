use crate::structures::{Assignment, ClauseId, Cnf, Literal};

#[derive(Debug)]
pub struct TrailSolve {
    cnf: Cnf,
    trail: Trail,
}

impl TrailSolve {
    pub fn new(cnf: Cnf) -> Self {
        let trail = Trail::for_cnf(&cnf);
        TrailSolve { cnf, trail }
    }

    pub fn find_unit(&self) -> Option<(Literal, ClauseId)> {
        for clause in self.cnf.clauses().iter() {
            if let Some(pair) = clause.get_unit_on(&self.trail.assignment) {
                return Some(pair);
            }
        }
        None
    }

    pub fn is_unsat(&self) -> bool {
        self.cnf.is_unsat_on(&self.trail.assignment)
    }

    pub fn is_sat(&self) -> bool {
        self.cnf.is_sat_on(&self.trail.assignment)
    }

    pub fn assume(&mut self, literal: Literal) {
        self.trail.set(literal, Source::Assumption);
    }
}

/// how a literal was added to an assignment
#[derive(Debug, PartialEq, Eq)]
enum Source {
    Choice,
    Deduction,
    Assumption,
}

/// a partial assignment with some history
// the assignment
#[derive(Debug)]
struct Trail {
    assignment: Assignment,
    history: Vec<(Literal, Source)>,
}

impl Trail {
    pub fn for_cnf(cnf: &Cnf) -> Self {
        Trail {
            assignment: Assignment::new(cnf.variables().len()),
            history: vec![],
        }
    }

    /// work back through steps of the trail, discarding the trail, and relaxing the assignment
    // here, some deduced literals may still hold, but for now the trail does not record multiple paths to a deduction
    pub fn track_back(&mut self, steps: usize) {
        for _step in 0..steps {
            if let Some((literal, _)) = self.history.pop() {
                self.assignment.clear(literal.variable())
            };
        }
    }

    pub fn backtrack_to_choice(&mut self) -> bool {
        let mut back_point = self.history.len() - 1;
        let mut back_steps = 0;
        while let Some((_, Source::Deduction)) = self.history.get(back_point) {
            back_point -= 1;
            back_steps += 1;
        }
        if back_steps != 0 {
            self.track_back(back_steps);
            true
        } else {
            false
        }
    }

    pub fn set(&mut self, literal: Literal, source: Source) {
        self.history.push((literal, source));
        self.assignment.set(literal);
    }
}
