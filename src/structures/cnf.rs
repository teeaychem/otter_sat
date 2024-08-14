use std::collections::BTreeSet;
use std::sync::atomic::{AtomicUsize, Ordering as AtomicOrdering};

use crate::structures::{Clause, ClauseError, Variable};

#[derive(Debug)]
pub enum CnfError {
    UnexpectedInformation,
    Clause(ClauseError),
}

#[derive(Debug)]
pub struct Cnf {
    variables: BTreeSet<Variable>,
    clauses: Vec<Clause>,
}

impl Cnf {
    pub fn new() -> Self {
        Cnf {
            variables: BTreeSet::new(),
            clauses: Vec::new(),
        }
    }

    pub fn variables(&self) -> &BTreeSet<Variable> {
        &self.variables
    }

    pub fn clauses(&self) -> &Vec<Clause> {
        &self.clauses
    }


    fn make_clause_id() -> usize {
        static COUNTER: AtomicUsize = AtomicUsize::new(1);
        COUNTER.fetch_add(1, AtomicOrdering::Relaxed)
    }

    /// adds a clause, taking ownership
    pub fn add_clause(&mut self, clause: Clause) -> bool {
        let mut owned_clause = clause;

        self.variables = self
            .variables
            .union(
                &owned_clause
                    .literals()
                    .iter()
                    .map(|l| l.variable())
                    .collect::<BTreeSet<_>>(),
            )
            .cloned()
            .collect();

        if owned_clause.id().is_none() {
            owned_clause.set_id(Self::make_clause_id());
        }

        self.clauses.push(owned_clause);

        false
    }
}
