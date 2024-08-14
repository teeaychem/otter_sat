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

    pub fn simple_solve(&mut self) -> bool {
        loop {
            // dbg!(&self.trail);
            if self.is_sat() {
                return true;
            } else if self.is_unsat() {
                if let Some(literal) = self.trail.undo_choice() {
                    self.trail.set(literal.negate(), Source::Deduction)
                } else {
                    return false;
                }
            } else if let Some((lit, _clause)) = self.find_unit() {
                self.trail.set(lit, Source::Deduction);
            } else if let Some(variable) = self.trail.assignment.get_unassigned() {
                if self.cnf.clauses().iter().any(|clause| {
                    clause
                        .literals()
                        .iter()
                        .filter(|l| l.variable() == variable)
                        .count()
                        > 0
                }) {
                    self.trail.set(
                        Literal::from_string(format!("{variable}").as_str()).expect("hek"),
                        Source::Choice,
                    );
                } else {
                    self.trail.set(
                        Literal::from_string(format!("{variable}").as_str()).expect("hek"),
                        Source::FreeChoice,
                    );
                }
            } else {
                panic!("A simple solve possibility has not been covered…");
            }
        }
    }
}

/// how a literal was added to an assignment
#[derive(Debug, PartialEq, Eq)]
enum Source {
    Choice,
    FreeChoice,
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
            assignment: Assignment::new(cnf.variables().len() + 1),
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

    pub fn backsteps_to_choice(&self) -> Option<usize> {
        let mut back_point = self.history.len() - 1;
        let mut back_steps = 0;
        loop {
            if let Some((_, source)) = self.history.get(back_point) {
                match source {
                    Source::Choice => return Some(back_steps),
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
            self.assignment.clear(literal.variable());
            Some(literal)
        } else {
            None
        }
    }

    pub fn set(&mut self, literal: Literal, source: Source) {
        self.history.push((literal, source));
        self.assignment.set(literal);
    }
}
