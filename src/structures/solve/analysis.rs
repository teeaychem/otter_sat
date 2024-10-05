use crate::procedures::{find_counterpart_literals, resolve_sorted_clauses};
use crate::structures::valuation::Valuation;
use crate::structures::{
    clause::{
        stored_clause::{initialise_watches_for, ClauseSource, StoredClause},
        Clause,
    },
    literal::{Literal, LiteralSource},
    solve::{
        config::{config_stopping_criteria, StoppingCriteria},
        the_solve::literal_update,
        Solve, SolveStatus,
    },
};

use std::collections::{BTreeSet, VecDeque};
use std::rc::Rc;

impl Solve {
    /// Either the most recent decision level in the resolution clause prior to the current level or 0.
    fn decision_level(&self, stored_clause: &StoredClause) -> usize {
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

    pub fn attempt_fix(&mut self, conflict_clause: &StoredClause) -> SolveStatus {
        let the_id = conflict_clause.id();
        log::trace!(
            "Attempt to fix on clause {the_id} at level {}",
            self.current_level().index()
        );
        match self.current_level().index() {
            0 => SolveStatus::NoSolution,
            _ => {
                let (asserting_clause, assertion) = self.conflict_analysis(conflict_clause);

                let backjump_level = self.decision_level(&asserting_clause);

                initialise_watches_for(
                    &asserting_clause,
                    &self.valuation_at(backjump_level),
                    &self.variables,
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
                literal_update(
                    assertion,
                    LiteralSource::StoredClause(asserting_clause),
                    &mut self.levels,
                    &self.variables,
                    &mut self.valuation,
                );
                self.watch_q.push_back(assertion);

                SolveStatus::AssertingClause
            }
        }
    }

    /// Simple analysis performs resolution on any clause used to obtain a conflict literal at the current decision
    pub fn conflict_analysis(
        &mut self,
        conflict_clause: &StoredClause,
    ) -> (Rc<StoredClause>, Literal) {
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

            if let LiteralSource::StoredClause(stored_clause) = src {
                let src_cls_vec = stored_clause.clause_impl();
                let counterparts =
                    find_counterpart_literals(resolved_clause.literals(), src_cls_vec.literals());

                if let Some(counterpart) = counterparts.first() {
                    resolution_trail.push(stored_clause.clone());
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
        let stored_clause =
            self.store_clause(resolved_clause, ClauseSource::Resolution(resolution_trail));
        stored_clause.set_lbd(&self.variables);

        (stored_clause, asserted_literal.unwrap())
    }

    pub fn core(&self) {
        println!();
        println!("c An unsatisfiable core of the original formula:\n");
        let node_indicies = self.levels[0]
            .observations()
            .iter()
            .filter_map(|(source, _)| match source {
                LiteralSource::StoredClause(weak) => Some(weak.clone()),
                _ => None,
            });
        let node_indicies_vec = node_indicies.collect::<Vec<_>>();
        let simple_core = extant_origins(node_indicies_vec);
        for clause in simple_core {
            println!("{}", clause.clause_impl().as_dimacs(&self.variables))
        }
        println!();
    }
}

pub fn extant_origins(clauses: Vec<Rc<StoredClause>>) -> Vec<Rc<StoredClause>> {
    #[allow(clippy::mutable_key_type)]
    let mut origin_nodes = BTreeSet::new();

    let mut q = VecDeque::new();
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
