use crate::structures::{
    Assignment, Clause, ClauseError, ClauseId, Literal, LiteralError, Variable, VariableId,
};


use std::sync::atomic::{AtomicUsize, Ordering as AtomicOrdering};

#[derive(Debug)]
pub struct Solve {
    variables: Vec<Variable>,
    pub clauses: Vec<Clause>,
}

impl Solve {
    pub fn new() -> Self {
        Solve {
            variables: Vec::new(),
            clauses: Vec::new(),
        }
    }

    pub fn v_id_from_name(&mut self, name: &str) -> VariableId {
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

        let the_variable = self.v_id_from_name(the_name);
        let the_literal = Literal::new(the_variable, polarity);
        Ok(the_literal)
    }

    pub fn v_by_id(&self, id: VariableId) -> Option<&Variable> {
        self.variables.get(id as usize)
    }
}

impl Solve {
    fn make_clause_id() -> ClauseId {
        static COUNTER: AtomicUsize = AtomicUsize::new(1);
        COUNTER.fetch_add(1, AtomicOrdering::Relaxed) as ClauseId
    }

    pub fn fresh_clause() -> Clause {
        Clause::new(Self::make_clause_id())
    }

    pub fn is_unsat_on(&self, assignment: &Assignment) -> bool {
        self.clauses
            .iter()
            .any(|clause| clause.is_unsat_on(assignment))
    }

    pub fn is_sat_on(&self, assignment: &Assignment) -> bool {
        self.clauses
            .iter()
            .all(|clause| clause.is_sat_on(assignment))
    }
}

#[derive(Debug)]
pub enum SolveError {
    Literal(LiteralError),
    Clause(ClauseError),
    ParseFailure,
    Hek,
}

impl Solve {
    pub fn find_unit_on(&self, assignment: &Assignment) -> Option<(Literal, ClauseId)> {
        for clause in self.clauses.iter() {
            if let Some(pair) = clause.get_unit_on(assignment) {
                return Some(pair);
            }
        }
        None
    }

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
                    the_trail.set(&literal.negate(), Source::Deduction)
                } else {
                    sat_assignment = Some((false, the_trail.assignment.clone()));
                    break;
                }
            }
            // 2. search
            if let Some((lit, _clause)) = the_trail.find_unit(self) {
                the_trail.set(&lit, Source::Deduction);
            } else if let Some(v_id) = the_trail.get_unassigned_id(self) {
                if self.clauses.iter().any(|clause| {
                    clause
                        .literals()
                        .iter()
                        .filter(|l| l.v_id() == v_id)
                        .count()
                        > 0
                }) {
                    the_trail.set(&Literal::new(v_id, true), Source::Choice);
                } else {
                    the_trail.set(&Literal::new(v_id, true), Source::FreeChoice);
                }
            } else {
                dbg!(the_trail);
                panic!("A simple solve possibility has not been covered…");
            }
        }
        match sat_assignment {
            Some(pair) => Ok(pair),
            None => Err(SolveError::Hek),
        }
    }
}

/// how a literal was added to an assignment
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
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
    pub fn for_solve(solve: &Solve) -> Self {
        Trail {
            assignment: Assignment::new(solve.variables.len()),
            history: vec![],
        }
    }

    /// work back through steps of the trail, discarding the trail, and relaxing the assignment
    // here, some deduced literals may still hold, but for now the trail does not record multiple paths to a deduction
    pub fn track_back(&mut self, steps: usize) {
        for _step in 0..steps {
            if let Some((literal, _)) = self.history.pop() {
                self.assignment.clear(literal.v_id())
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
            self.assignment.clear(literal.v_id());
            Some(literal)
        } else {
            None
        }
    }

    pub fn set(&mut self, literal: &Literal, source: Source) {
        self.history.push((literal.clone(), source));
        self.assignment.set(literal.clone());
    }

    pub fn get_unassigned_id(&self, solve: &Solve) -> Option<VariableId> {
        solve
            .variables
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
            if let Some(pair) = clause.get_unit_on(&self.assignment) {
                return Some(pair);
            }
        }
        None
    }
}
