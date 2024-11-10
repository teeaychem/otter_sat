use std::{borrow::Borrow, sync::mpsc::Sender};

use crate::{
    context::{stores::ClauseKey, Context, SolveStatus},
    structures::{
        literal::{Literal, LiteralSource, LiteralTrait},
        variable::list::VariableList,
    },
    types::{clause::ClauseSource, errs::ClauseStoreErr},
    FRAT::FRATStep,
};

use super::unique_id::UniqueIdentifier;
use crate::context::unique_id::UniqueId;

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
        resolution_keys: Vec<UniqueIdentifier>,
    ) -> Result<ClauseKey, ClauseStoreErr> {
        self.clause_store.insert_clause(
            source,
            clause,
            &mut self.variables,
            &mut self.traces,
            resolution_keys,
            &self.config,
        )
    }

    pub fn note_literal<L: Borrow<impl LiteralTrait>>(
        &mut self,
        literal: L,
        source: LiteralSource,
        resolution_keys: Vec<UniqueIdentifier>,
    ) {
        let canonical = literal.borrow().canonical();

        self.levels.record_literal(canonical, source);

        if self.config.io.frat_path.is_some() {
            // Only record…
            let step = match source {
                // … resolution instances which led to a unit asserting clause
                LiteralSource::Resolution(_) => Some(FRATStep::learnt_literal(
                    canonical,
                    &resolution_keys,
                    &self.variables,
                )),
                // … unit clauses of the original formula reinterpreted as assumptions
                LiteralSource::Assumption => {
                    Some(FRATStep::original_literal(canonical, &self.variables))
                }
                // … and nothing else
                _ => None,
            };
            if let Some(made_step) = step {
                self.traces.frat.record(made_step);
                self.traces
                    .serial
                    .push((literal.borrow().canonical().unique_id(), resolution_keys));
            }
        }
    }

    pub fn print_status(&self, tx: Sender<String>) {
        if self.config.io.show_stats {
            if let Some(window) = &self.window {
                window.update_counters(&self.counters);
                window.flush();
            }
        }

        match self.status {
            SolveStatus::FullValuation => {
                let _ = tx.send("s SATISFIABLE".to_string());
                if self.config.io.show_valuation {
                    let _ = tx.send(format!("v {}", self.valuation_string()));
                }
            }
            SolveStatus::NoSolution => {
                let _ = tx.send("s UNSATISFIABLE".to_string());
                if self.config.io.show_core {
                    // let _ = self.display_core(clause_key);
                }
            }
            SolveStatus::NoClauses => {
                if self.config.io.detail > 0 {
                    let _ = tx.send(
                        "c The formula contains no clause and so is interpreted as ⊤".to_string(),
                    );
                }
                let _ = tx.send("s SATISFIABLE".to_string());
            }
            _ => {
                if let Some(limit) = self.config.time_limit {
                    if self.config.io.show_stats && self.counters.time > limit {
                        let _ = tx.send("c TIME LIMIT EXCEEDED".to_string());
                    }
                }
                let _ = tx.send("s UNKNOWN".to_string());
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

    pub fn print_valuation(&self) {
        println!("v {:?}", self.valuation_string());
    }
}
