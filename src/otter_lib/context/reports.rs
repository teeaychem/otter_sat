use std::ops::Deref;

use crate::{
    context::{stores::ClauseKey, Context},
    structures::{
        clause::Clause,
        literal::{LiteralSource, LiteralTrait},
    },
    types::errs::ReportError,
    FRAT::FRATStep,
};

use super::{delta::SolveReport, SolveStatus};

// FRAT
impl Context {
    pub fn frat_formula(&mut self) {
        for formula in self.clause_store.formula_clauses() {
            self.traces.frat.record(FRATStep::original_clause(
                formula.key(),
                formula.deref(),
                &self.variables,
            ))
        }
        self.traces.frat.flush(&self.config)
    }

    // TODO: finalise
    pub fn frat_finalise(&mut self) {
        for formula in self.clause_store.all_clauses() {
            self.traces.frat.record(FRATStep::finalise(
                formula.key(),
                formula.deref(),
                &self.variables,
            ))
        }
        self.traces.frat.flush(&self.config)
    }
}

impl Context {
    pub fn report(&self) -> SolveReport {
        match self.status {
            SolveStatus::FullValuation => SolveReport::Satisfiable,
            SolveStatus::NoClauses => SolveReport::Satisfiable,
            SolveStatus::NoSolution => SolveReport::Unsatisfiable,
            _ => SolveReport::Unknown,
        }
    }

    //     #[allow(clippy::single_match)]
    //     /// An unsatisfiable core
    //     pub fn get_unsat_core(&self, conflict_key: ClauseKey) -> Result<Vec<ClauseKey>, ReportError> {
    //         let Report::Unsatisfiable = self.report() else {
    //             return Err(ReportError::UnsatCoreUnavailable);
    //         };

    //         println!("c An unsatisfiable core of the formula:\n",);

    //         /*
    //         Given the conflict clause, collect the following:

    //         - The formula clauses used to resolve the conflict clause
    //         - The formula clauses used to establish any literal whose negation appears in some considered clause

    //         The core_q queues clause keys for inspection
    //         The seen literal set helps to avoid checking the same literal twice
    //         Likewise, the key set helps to avoid checking the same key twice
    //          */
    //         let mut core_q = std::collections::VecDeque::<ClauseKey>::new();
    //         let mut seen_literal_set = std::collections::BTreeSet::new();
    //         let mut key_set = std::collections::BTreeSet::new();
    //         let mut core_keys = std::collections::BTreeSet::new();

    //         // for short arguments
    //         let observations = self.levels.get(0).observations();

    //         // start with the conflict, then loop
    //         core_q.push_back(conflict_key);

    //         /*
    //         key set ensures processing only happens on a fresh key

    //         if the key is for a formula, then clause is recorded and the literals of the clause are checked against the observed literals
    //         otherwise, the clauses used when resolving the learnt clause are added

    //          when checking literals, if the negation of the literal has been observed at level 0 then it was relevant to the conflict
    //          so, if the literal was obtained either by resolution or directly from some clause, then that clause or the clauses used for resolution are added to the q
    //          this skips assumed literals
    //          */
    //         while let Some(key) = core_q.pop_front() {
    //             if key_set.insert(key) {
    //                 match key {
    //                     ClauseKey::Formula(_) => {
    //                         let clause = self.clause_store.get(key)?;

    //                         core_keys.insert(key);

    //                         for literal in clause.deref() {
    //                             if seen_literal_set.insert(*literal) {
    //                                 let found = observations.iter().find(|(_, observed_literal)| {
    //                                     literal == &observed_literal.negate()
    //                                 });
    //                                 if let Some((src, _)) = found {
    //                                     match src {
    //                                         LiteralSource::Resolution(_) => {
    //                                             let proof = &self
    //                                                 .proofs
    //                                                 .iter()
    //                                                 .find(|(proven_literal, _)| {
    //                                                     literal == &proven_literal.negate()
    //                                                 })
    //                                                 .expect("no proof of resolved literal");
    //                                             for key in &proof.1 {
    //                                                 core_q.push_back(*key);
    //                                             }
    //                                         }
    //                                         LiteralSource::Analysis(clause_key)
    //                                         | LiteralSource::BCP(clause_key)
    //                                         | LiteralSource::Missed(clause_key) => {
    //                                             core_q.push_back(*clause_key)
    //                                         }

    //                                         LiteralSource::Choice
    //                                         | LiteralSource::Pure
    //                                         | LiteralSource::Assumption => {}
    //                                     }
    //                                 }
    //                             }
    //                         }
    //                     }
    //                     ClauseKey::Binary(_) | ClauseKey::Learned(_, _) => {
    //                         let source = self.clause_store.source(key);
    //                         for source_key in source {
    //                             core_q.push_back(*source_key);
    //                         }
    //                     }
    //                 }
    //             }
    //         }

    //         Ok(core_keys.into_iter().collect())
    //     }

    //     pub fn display_core(&self, conflict_key: ClauseKey) -> Result<(), ReportError> {
    //         let the_core = self.get_unsat_core(conflict_key)?;
    //         for key in the_core {
    //             let clause = self.clause_store.get(key)?;
    //             println!("{}", clause.as_dimacs(&self.variables));
    //         }
    //         Ok(())
    //     }
}

impl Context {
    pub fn clause_database(&self) -> Vec<String> {
        self.clause_store
            .all_clauses()
            .map(|clause| clause.as_dimacs(&self.variables, true))
            .collect::<Vec<_>>()
    }

    pub fn proven_literal_database(&self) -> Vec<String> {
        self.levels
            .proven_literals()
            .iter()
            .map(|literal| format!("{} 0", self.variables.external_name(literal.index())))
            .collect::<Vec<_>>()
    }
}
