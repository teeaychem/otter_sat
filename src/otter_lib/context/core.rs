use std::borrow::Borrow;

use crate::{
    context::{stores::ClauseKey, Context, SolveStatus},
    dispatch::{
        self,
        delta::{self},
        Dispatch,
    },
    structures::{
        literal::{Literal, LiteralSource, LiteralTrait},
        variable::list::VariableList,
    },
    types::{clause::ClauseSource, errs::ClauseStoreErr},
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
    ) -> Result<ClauseKey, ClauseStoreErr> {
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

        self.levels.record_literal(canonical, source);

        // Only record…
        match source {
            // … resolution instances which led to a unit asserting clause
            LiteralSource::Resolution(_) => {
                self.sender
                    .send(Dispatch::Level(delta::Level::ResolutionProof(canonical)));
            }
            // … unit clauses of the original formula reinterpreted as assumptions
            LiteralSource::Assumption => {
                self.sender
                    .send(Dispatch::Level(delta::Level::FormulaAssumption(canonical)));
            }
            // … and nothing else
            _ => {}
        }
    }

    pub fn print_status(&self) {
        if self.config.io.show_stats {
            if let Some(window) = &self.window {
                window.update_counters(&self.counters);
                window.flush();
            }
        }

        match self.status {
            SolveStatus::FullValuation => {
                let _ = self
                    .sender
                    .send(Dispatch::SolveReport(dispatch::SolveReport::Satisfiable));
            }
            SolveStatus::NoSolution => {
                let _ = self
                    .sender
                    .send(Dispatch::SolveReport(dispatch::SolveReport::Unsatisfiable));
                if self.config.io.show_core {
                    // let _ = self.display_core(clause_key);
                }
            }
            SolveStatus::NoClauses => {
                if self.config.io.detail > 0 {
                    let _ = self
                        .sender
                        .send(Dispatch::SolveComment(dispatch::SolveComment::NoClauses));
                }
                let _ = self
                    .sender
                    .send(Dispatch::SolveReport(dispatch::SolveReport::Satisfiable));
            }
            _ => {
                let _ = self
                    .sender
                    .send(Dispatch::SolveReport(dispatch::SolveReport::Unknown));
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
}
