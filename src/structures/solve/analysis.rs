use crate::procedures::{find_counterpart_literals, resolve_sorted_clauses};
use crate::structures::valuation::Valuation;
use crate::structures::{
    clause::{
        clause_vec::ClauseVec,
        stored_clause::{initialise_watches_for, ClauseSource, StoredClause, Watch},
        Clause,
    },
    literal::{Literal, LiteralSource},
    solve::{
        clause_store::{retreive, ClauseKey},
        config::{config_stopping_criteria, StoppingCriteria},
        the_solve::literal_update,
        Solve, SolveStatus,
    },
    variable::Variable,
};

use std::collections::VecDeque;

impl Solve {
    pub fn attempt_fix(&mut self, clause_key: ClauseKey) -> SolveStatus {
        let conflict_clause = retreive(&self.clauses_stored, clause_key);

        log::trace!(
            "Attempt to fix on clause {conflict_clause} at level {}",
            self.current_level().index()
        );
        match self.current_level().index() {
            0 => SolveStatus::NoSolution,
            _ => {
                let (asserting_clause, clause_source, assertion) =
                    self.conflict_analysis(conflict_clause);

                let clause_key = self.store_clause(asserting_clause, clause_source);
                let stored_clause = retreive(&self.clauses_stored, clause_key);

                let anticipated_literal_source = LiteralSource::StoredClause(stored_clause.key);

                stored_clause.set_lbd(&self.variables);

                let backjump_level = decision_level(&self.variables, stored_clause);

                let backjump_valuation = self.valuation_at(backjump_level);
                initialise_watches_for(stored_clause, &backjump_valuation, &self.variables);

                if assertion == stored_clause.literal_of(Watch::A) {
                    self.watch_q
                        .push_back(stored_clause.literal_of(Watch::B).negate());
                } else if assertion == stored_clause.literal_of(Watch::B) {
                    self.watch_q
                        .push_back(stored_clause.literal_of(Watch::A).negate());
                } else {
                    panic!("Failed to predict asserting clause")
                }

                self.backjump(backjump_level);

                // updating the valuation needs to happen here to ensure the watches for any queued literal during propagaion are fixed
                literal_update(
                    assertion,
                    anticipated_literal_source,
                    &mut self.levels,
                    &self.variables,
                    &mut self.valuation,
                    &self.clauses_stored,
                );
                self.watch_q.push_back(assertion);

                SolveStatus::AssertingClause
            }
        }
    }

    /// Simple analysis performs resolution on any clause used to obtain a conflict literal at the current decision
    pub fn conflict_analysis(
        &self,
        conflict_clause: &StoredClause,
    ) -> (ClauseVec, ClauseSource, Literal) {
        let mut resolved_clause = conflict_clause.clause_clone();
        let mut resolution_trail = vec![];

        let previous_level_val = self.valuation_at(self.current_level().index() - 1);
        let mut asserted_literal = None;

        let stopping_criteria = config_stopping_criteria();
        for (src, _lit) in self.current_level().observations().iter().rev() {
            match stopping_criteria {
                StoppingCriteria::FirstAssertingUIP => {
                    if let Some(asserted) = resolved_clause.asserts(&previous_level_val) {
                        asserted_literal = Some(asserted);
                        break;
                    }
                }
                StoppingCriteria::None => (),
            }

            if let LiteralSource::StoredClause(clause_key) = src {
                let stored_clause = retreive(&self.clauses_stored, *clause_key);
                let src_cls_vec = stored_clause.clause_impl();
                let counterparts =
                    find_counterpart_literals(resolved_clause.literals(), src_cls_vec.literals());

                if let Some(counterpart) = counterparts.first() {
                    resolution_trail.push(*clause_key);
                    resolved_clause = resolve_sorted_clauses(
                        resolved_clause.literals(),
                        src_cls_vec.literals(),
                        *counterpart,
                    )
                    .unwrap()
                    .to_vec();
                }
            }
        }

        if asserted_literal.is_none() {
            if let Some(asserted) = resolved_clause.asserts(&previous_level_val) {
                asserted_literal = Some(asserted);
            } else {
                println!("PV {}", previous_level_val.as_internal_string());
                println!("CV {}", self.valuation.as_internal_string());
                panic!("No assertion…")
            }
        }

        /*
        If some literals are known then their negation can be safely removed from the learnt clause.
        Though, this isn't a particular effective method…
         */
        if !self.levels[0].observations().is_empty() {
            resolved_clause.retain(|l| {
                !self.levels[0]
                    .observations()
                    .iter()
                    .any(|(_, x)| l.negate() == *x)
            })
        }

        (
            resolved_clause,
            ClauseSource::Resolution(resolution_trail),
            asserted_literal.unwrap(),
        )
    }

    pub fn core(&self) {
        println!();
        println!("c An unsatisfiable core of the original formula:\n");
        let node_indicies = self.levels[0]
            .observations()
            .iter()
            .filter_map(|(source, _)| match source {
                LiteralSource::StoredClause(weak) => Some(*weak),
                _ => None,
            });
        let node_indicies_vec = node_indicies.collect::<Vec<_>>();
        let simple_core = self.extant_origins(node_indicies_vec);
        for clause in simple_core {
            println!("{}", clause.clause_impl().as_dimacs(&self.variables))
        }
        println!();
    }

    pub fn extant_origins(&self, clauses: Vec<ClauseKey>) -> Vec<&StoredClause> {
        #[allow(clippy::mutable_key_type)]
        let mut origin_nodes = vec![];

        let mut q = VecDeque::new();
        for clause in clauses {
            q.push_back(clause);
        }
        loop {
            if q.is_empty() {
                break;
            }

            let clause_key = q.pop_front().expect("Ah, the queue was empty…");
            let stored_clause = retreive(&self.clauses_stored, clause_key);

            match stored_clause.source() {
                ClauseSource::Resolution(origins) => {
                    for antecedent in origins {
                        q.push_back(*antecedent);
                    }
                }
                ClauseSource::Formula => {
                    origin_nodes.push(stored_clause);
                }
            }
        }
        origin_nodes
    }
}

/// Either the most recent decision level in the resolution clause prior to the current level or 0.
fn decision_level(variables: &[Variable], stored_clause: &StoredClause) -> usize {
    let mut top_two = [None; 2];
    for lit in stored_clause.literals() {
        if let Some(dl) = variables[lit.v_id].decision_level() {
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
