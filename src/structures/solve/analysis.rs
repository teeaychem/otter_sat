use crate::procedures::{find_counterpart_literals, resolve_sorted_clauses};
use crate::structures::solve::mutation::process_update_literal;
use crate::structures::solve::mutation::process_variable_occurrence_update;
use crate::structures::solve::{Solve, SolveError, SolveOk};
use crate::structures::valuation::Valuation;
use crate::structures::{
    solve::StoppingCriteria, stored_clause::initialise_watches_for, Clause, ClauseSource, Literal,
    LiteralSource, StoredClause,
};

use std::collections::{BTreeSet, VecDeque};
use std::rc::Rc;

pub enum AnalysisResult {
    AssertingClause(Rc<StoredClause>, Literal),
}

impl Solve<'_> {
    /// Either the most recent decision level in the resolution clause prior to the current level or 0.
    fn decision_level(&self, stored_clause: &Rc<StoredClause>) -> usize {
        let mut top_two = [None; 2];
        for lit in stored_clause.literals() {
            if let Some(dl) = self.variables[lit.v_id].decision_level() {
                if top_two[1].is_none() {
                    top_two[1] = Some(dl)
                } else if top_two[1].is_some_and(|t1| dl > t1) {
                    top_two[0] = top_two[1];
                    top_two[1] = Some(dl)
                } else if top_two[0].is_none() || top_two[0].is_some_and(|t2| dl > t2) {
                    top_two[0] = Some(dl)
                };
            }
        }

        match top_two {
            [None, Some(_)] => 0,
            [Some(x), Some(_)] => x,
            _ => panic!("Decision level issue: {:?}", top_two),
        }
    }

    pub fn attempt_fix(
        &mut self,
        conflict_clause: Rc<StoredClause>,
    ) -> Result<SolveOk, SolveError> {
        let the_id = conflict_clause.id();
        log::warn!(
            "Attempt to fix on clause {the_id} at level {}",
            self.current_level().index()
        );
        match self.current_level().index() {
            0 => Err(SolveError::NoSolution),
            _ => match self.conflict_analysis(conflict_clause) {
                AnalysisResult::AssertingClause(asserting_clause, assertion) => {
                    let backjump_level = self.decision_level(&asserting_clause);

                    initialise_watches_for(
                        &asserting_clause,
                        &self.valuation_at(backjump_level),
                        &mut self.variables,
                    );

                    if assertion == asserting_clause.watched_a() {
                        self.watch_q
                            .push_back(asserting_clause.watched_b().negate());
                    } else if assertion == asserting_clause.watched_b() {
                        self.watch_q
                            .push_back(asserting_clause.watched_a().negate());
                    } else {
                        panic!("Failed to predict asserting clause")
                    }

                    self.backjump(backjump_level);

                    // updating the valuation needs to happen here to ensure the watches for any queued literal during propagaion are fixed
                    let valuation_result = self.valuation.update_value(assertion);
                    let _chose_literal_without_value = process_update_literal(
                        assertion,
                        LiteralSource::StoredClause(asserting_clause),
                        &mut self.variables[assertion.v_id],
                        &mut self.levels,
                        valuation_result,
                    );
                    if process_variable_occurrence_update(
                        &self.valuation,
                        &mut self.variables,
                        assertion,
                    ) {
                        self.watch_q.push_back(assertion);
                    }

                    Ok(SolveOk::AssertingClause)
                }
            },
        }
    }

    pub fn attempt_fixes(
        &mut self,
        conflict_clauses: Vec<Rc<StoredClause>>,
    ) -> Result<SolveOk, SolveError> {
        if self.current_level().index() == 0 {
            log::warn!("Base level fix…");
            return Err(SolveError::NoSolution);
        } else {
            let mut analysis_results = vec![];

            let mut the_jump = if self.config.multi_jump_max {
                usize::MAX
            } else {
                usize::MIN
            };

            for conflict_clause in conflict_clauses {
                match self.conflict_analysis(conflict_clause) {
                    AnalysisResult::AssertingClause(asserting_clause, _) => {
                        let backjump_level = self.decision_level(&asserting_clause);
                        if (self.config.multi_jump_max && backjump_level < the_jump)
                            || backjump_level > the_jump
                        {
                            the_jump = backjump_level
                        }
                        analysis_results.push(asserting_clause);
                    }
                }
            }

            let the_valuation = self.valuation_at(the_jump);

            for asserting_clause in analysis_results {
                initialise_watches_for(&asserting_clause, &the_valuation, &mut self.variables);
            }

            self.backjump(the_jump);
        }

        Ok(SolveOk::AssertingClause)
    }

    pub fn analysis_switch(&mut self, conflict_clause: Rc<StoredClause>) -> AnalysisResult {
        match self.config.analysis {
            3 => self.conflict_analysis(conflict_clause),
            _ => panic!("Unknown analysis"),
        }
    }

    /// Simple analysis performs resolution on any clause used to obtain a conflict literal at the current decision
    pub fn conflict_analysis(&mut self, conflict_clause: Rc<StoredClause>) -> AnalysisResult {
        let mut resolved_clause = conflict_clause.as_vec();
        let mut resolution_trail = vec![];

        let previous_level_val = self.valuation_at(self.current_level().index() - 1);
        let mut asserted_literal = None;

        for (src, _lit) in self.current_level().observations().iter().rev() {
            match self.config.stopping_criteria {
                StoppingCriteria::FirstAssertingUIP => {
                    if let Some(asserted) = resolved_clause.asserts(&previous_level_val) {
                        asserted_literal = Some(asserted);
                        break;
                    }
                }
                StoppingCriteria::None => (),
            }

            if let LiteralSource::StoredClause(stored_clause) = src {
                let src_cls_vec = stored_clause.as_vec();
                let counterparts = find_counterpart_literals(&resolved_clause, &src_cls_vec);

                if let Some(counterpart) = counterparts.first() {
                    resolution_trail.push(stored_clause.clone());
                    resolved_clause =
                        resolve_sorted_clauses(&resolved_clause, &src_cls_vec, *counterpart)
                            .unwrap()
                            .to_vec()
                }
            }
        }

        /*
        If some literals are known then their negation can be safely removed from the learnt clause.
        Though, this isn't a particular effective method…
         */
        if !self.top_level().observations().is_empty() {
            resolved_clause.retain(|l| {
                !self
                    .top_level()
                    .observations()
                    .iter()
                    .any(|(_, x)| l.negate() == *x)
            })
        }
        let stored_clause = self.store_clause(
            resolved_clause.clone(),
            ClauseSource::Resolution(resolution_trail),
        );
        stored_clause.set_lbd(&self.variables);

        if let Some(asserted) = asserted_literal {
            AnalysisResult::AssertingClause(stored_clause, asserted)
        } else if let Some(asserted) = resolved_clause.asserts(&previous_level_val) {
            AnalysisResult::AssertingClause(stored_clause, asserted)
        } else {
            panic!("No assertion…")
        }
    }

    pub fn core(&self) {
        println!();
        println!("c An unsatisfiable core of the original formula:\n");
        let node_indicies = self
            .top_level()
            .observations()
            .iter()
            .filter_map(|(source, _)| match source {
                LiteralSource::StoredClause(weak) => Some(weak.clone()),
                _ => None,
            });
        let node_indicies_vec = node_indicies.collect::<Vec<_>>();
        let simple_core = extant_origins(node_indicies_vec);
        for clause in simple_core {
            println!("{}", clause.clause().as_dimacs(&self.variables))
        }
        println!();
    }
}

pub fn extant_origins(clauses: Vec<Rc<StoredClause>>) -> Vec<Rc<StoredClause>> {
    #[allow(clippy::mutable_key_type)]
    let mut origin_nodes = BTreeSet::new();

    let mut q: VecDeque<Rc<StoredClause>> = VecDeque::new();
    for clause in clauses {
        q.push_back(clause);
    }
    loop {
        if q.is_empty() {
            break;
        }

        let stored_clause = q.pop_front().expect("Ah, the queue was empty…");

        match stored_clause.source() {
            ClauseSource::Resolution(origins) => {
                for antecedent in origins {
                    q.push_back(antecedent.clone());
                }
            }
            ClauseSource::Formula => {
                origin_nodes.insert(stored_clause.clone());
            }
        }
    }
    origin_nodes.into_iter().collect::<Vec<_>>()
}
