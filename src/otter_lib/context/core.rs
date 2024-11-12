use std::borrow::Borrow;

use crate::{
    context::{stores::ClauseKey, Context, SolveStatus},
    dispatch::{
        self,
        comment::{self},
        delta::{self},
        report::{self},
        Dispatch,
    },
    structures::{
        literal::{Literal, LiteralSource, LiteralTrait},
        variable::list::VariableList,
    },
    types::{clause::ClauseSource, errs::ClauseDB},
};

#[derive(Debug, Clone, Copy)]
pub enum StepInfo {
    Conflict,
    ChoicesExhausted,
    ChoiceMade,
    One,
}

#[derive(Debug)]
pub enum ContextFailure {
    QueueConflict,
}

impl Context {
    pub fn variable_count(&self) -> usize {
        self.variables.len()
    }

    pub fn clause_count(&self) -> usize {
        self.clause_store.clause_count()
    }

    /// Stores a clause with an automatically generated id.
    /// In order to use the clause the watch literals of the struct must be initialised.
    pub fn store_clause(
        &mut self,
        clause: Vec<Literal>,
        source: ClauseSource,
        resolution_keys: Vec<ClauseKey>,
    ) -> Result<ClauseKey, ClauseDB> {
        self.clause_store.insert_clause(
            source,
            clause,
            &mut self.variables,
            resolution_keys,
            &self.config,
        )
    }

    pub fn note_literal<L: Borrow<impl LiteralTrait>>(
        &mut self,
        literal: L,
        source: LiteralSource,
        resolution_keys: Vec<ClauseKey>,
    ) {
        let canonical = literal.borrow().canonical();
        log::trace!("Noted {}", canonical);
        self.levels.record_literal(canonical, source);
    }

    pub fn print_status(&self) {
        // if self.config.io.show_stats {
        //     if let Some(window) = &self.window {
        //         window.update_counters(&self.counters);
        //         window.flush();
        //     }
        // }

        match self.status {
            SolveStatus::FullValuation => {
                let _ = self
                    .tx
                    .send(Dispatch::SolveReport(report::Solve::Satisfiable));
            }
            SolveStatus::NoSolution => {
                let report = report::Solve::Unsatisfiable;
                let _ = self.tx.send(Dispatch::SolveReport(report));
            }
            SolveStatus::NoClauses => {
                self.tx
                    .send(Dispatch::SolveComment(comment::Solve::NoClauses));
            }
            _ => {
                let _ = self.tx.send(Dispatch::SolveReport(report::Solve::Unknown));
            }
        }
    }

    pub fn valuation_string(&self) -> String {
        self.variables
            .slice()
            .iter()
            .filter_map(|v| match v.value() {
                None => None,
                Some(true) => Some(format!(" {}", self.variables.external_name(v.index()))),
                Some(false) => Some(format!("-{}", self.variables.external_name(v.index()))),
            })
            .collect::<Vec<_>>()
            .join(" ")
    }

    pub fn internal_valuation_string(&self) -> String {
        let mut v = self
            .variables
            .slice()
            .iter()
            .enumerate()
            .filter_map(|(i, v)| match v.value() {
                None => None,
                Some(true) => Some(i as isize),
                Some(false) => Some(-(i as isize)),
            })
            .collect::<Vec<_>>();
        v.sort_unstable();
        v.iter()
            .map(|v| v.to_string())
            .collect::<Vec<_>>()
            .join(" ")
    }

    pub fn report_active(&self) {
        for clause in self.clause_store.all_clauses() {
            if clause.is_active() {
                let report = report::ClauseDB::Active(clause.key(), clause.to_vec());
                self.tx.send(Dispatch::ClauseDBReport(report));
            }
        }
        for literal in self.levels.proven_literals() {
            let report = report::VariableDB::Active(*literal);
            self.tx.send(Dispatch::VariableDBReport(report));
        }
    }
}
