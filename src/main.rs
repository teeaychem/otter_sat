use std::{cmp::Ordering, collections::BTreeSet};

fn main() {
    println!("Hello, world!");
}

type Variable = usize;

#[derive(Clone, Copy)]
struct Literal {
    variable: Variable,
    polarity: bool,
}

impl PartialOrd for Literal {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Literal {
    fn cmp(&self, other: &Self) -> Ordering {
        if self.variable == other.variable {
            if self.polarity == other.polarity {
                Ordering::Equal
            } else if self.polarity {
                Ordering::Less
            } else {
                Ordering::Greater
            }
        } else {
            self.variable.cmp(&other.variable)
        }
    }
}

impl PartialEq for Literal {
    fn eq(&self, other: &Self) -> bool {
        self.variable == other.variable && self.polarity == other.polarity
    }
}

impl Eq for Literal {}

type ClauseId = usize;

struct Clause {
    id: ClauseId,
    literals: Vec<Literal>,
}

impl Clause {
    ///
    pub fn get_unit_on(&self, assignment: &Assignment) -> Option<(Literal, ClauseId)> {
        let mut unit = None;
        for literal in &self.literals {
            if let Some(assignment) = assignment.get(literal.variable) {
                match assignment {
                    Some(true) => break,     // as the clause does not provide any new information
                    Some(false) => continue, // some other literal must be true
                    None => {
                        // if no assignment, then literal must be true…
                        match unit {
                            Some(_) => {
                                // æbut if there was already a literal, it's not implied
                                unit = None;
                                break;
                            }
                            None => unit = Some((*literal, self.id)), // still, if everything so far is false, this literal must be true, for now…
                        }
                    }
                }
            }
        }
        unit
    }
}

struct Cnf {
    variables: usize,
    clauses: Vec<Clause>,
}

/// how a literal was added to an assignment
#[derive(PartialEq, Eq)]
enum Source {
    Choice,
    Deduction,
}

type Assignment = Vec<Option<bool>>;

/// a partial assignment with some history
// the assignment
struct Trail {
    assignment: Assignment,
    history: Vec<(Literal, Source)>,
}

impl Trail {
    /// work back through steps of the trail, discarding the trail, and relaxing the assignment
    // here, some deduced literals may still hold, but for now the trail does not record multiple paths to a deduction
    pub fn track_back(&mut self, steps: usize) {
        for _step in 0..steps {
            if let Some((literal, _)) = self.history.pop() {
                self.assignment[literal.variable] = None
            };
        }
    }
}

impl Trail {
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
}

struct Solve {
    cnf: Cnf,
    trail: Trail,
}
