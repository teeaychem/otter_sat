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

struct Clause {
    literals: Vec<Literal>,
}

impl Clause {
    pub fn is_unit_on(&self, trail: &Trail) -> bool {
        self.literals
            .iter()
            .filter(|&l| !trail.literals.contains(l))
            .count()
            == 1
    }
}

struct Cnf {
    clauses: BTreeSet<Clause>,
}

/// how a literal was added to an assignment
#[derive(PartialEq, Eq)]
enum Source {
    Choice,
    Deduction,
}

/// a partial assignment with some history
struct Trail {
    literals: Vec<Literal>,
    history: Vec<Source>,
}

impl Trail {
    pub fn backtrack_to_choice(&mut self) -> bool {
        let mut back_point = self.history.len() - 1;
        while let Some(Source::Deduction) = self.history.get(back_point) {
            back_point -= 1;
        }
        if back_point != self.history.len() {
            self.history.truncate(back_point);
            self.literals.truncate(back_point);
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
