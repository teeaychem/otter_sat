use crate::structures::{
    Assignment, Clause, ClauseError, ClauseId, Literal, LiteralError, Variable, VariableId,
};
use std::collections::BTreeSet;

use std::sync::atomic::{AtomicUsize, Ordering as AtomicOrdering};

#[derive(Debug)]
pub struct Solve {
    variables: BTreeSet<Variable>,
    pub clauses: Vec<Clause>,
}

impl Solve {
    pub fn new() -> Self {
        Solve {
            variables: BTreeSet::new(),
            clauses: Vec::new(),
        }
    }

    fn make_literal_id() -> VariableId {
        static COUNTER: AtomicUsize = AtomicUsize::new(1);
        COUNTER.fetch_add(1, AtomicOrdering::Relaxed) as u32
    }

    pub fn variable_by_name(&mut self, name: &str) -> Variable {
        if let Some(variable) = self.variables.iter().find(|v| v.name == name) {
            variable.clone()
        } else {
            let new_variable = Variable {
                name: name.to_string(),
                id: Self::make_literal_id(),
            };
            self.variables.insert(new_variable.clone());
            new_variable
        }
    }

    pub fn literal_from_string(&mut self, string: &str) -> Result<Literal, SolveError> {
        println!("| literal from string: {}", string);
        let trimmed_string = string.trim();

        if trimmed_string.is_empty() || trimmed_string == "-" {
            return Err(SolveError::Literal(LiteralError::NoVariable));
        }

        let polarity = trimmed_string.chars().nth(0) != Some('-');

        let mut the_name = trimmed_string;
        if !polarity {
            the_name = &the_name[1..]
        }
        println!("| name: {}", the_name);

        let the_variable = self.variable_by_name(the_name);
        println!("| variable: {:?}", the_variable);
        let the_literal = Literal::new(the_variable, polarity);
        Ok(the_literal)
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

    pub fn simple_solve(&mut self) -> Result<bool, SolveError> {
        let mut the_trail = Trail::for_solve(self);

        loop {
            if self.is_sat_on(&the_trail.assignment) {
                return Ok(true);
            } else if self.is_unsat_on(&the_trail.assignment) {
                if let Some(literal) = the_trail.undo_choice() {
                    the_trail.set(&literal.negate(), Source::Deduction)
                } else {
                    return Ok(false);
                }
            } else if let Some((lit, _clause)) = the_trail.find_unit(self) {
                the_trail.set(&lit, Source::Deduction);
            } else if let Some(variable) = the_trail.get_unassigned(self) {
                println!("unassignemd: {:?}", variable);
                if self.clauses.iter().any(|clause| {
                    clause
                        .literals()
                        .iter()
                        .filter(|l| l.variable() == &variable)
                        .count()
                        > 0
                }) {
                    the_trail.set(&Literal::new(variable, true), Source::Choice);
                } else {
                    the_trail.set(&Literal::new(variable, true), Source::FreeChoice);
                }
            } else {
                dbg!(the_trail);
                panic!("A simple solve possibility has not been covered…");
            }
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
            assignment: Assignment::new(solve.variables.len() + 1),
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

    pub fn set(&mut self, literal: &Literal, source: Source) {
        self.history.push((literal.clone(), source));
        self.assignment.set(literal.clone());
    }

    pub fn get_unassigned(&self, solve: &Solve) -> Option<Variable> {
        solve
            .variables
            .iter()
            .find(|&v| self.assignment.get(v).is_ok_and(|x| x.is_none()))
            .cloned()
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
