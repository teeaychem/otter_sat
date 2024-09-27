use crate::procedures::{find_counterpart_literals, resolve_sorted_clauses};
use crate::structures::solve::{Solve, SolveError, SolveOk};
use crate::structures::{
    stored_clause::initialise_watches_for, Clause, ClauseSource, ClauseVec, LiteralSource,
    StoredClause,
};

use std::collections::{BTreeSet, VecDeque};
use std::rc::{Rc, Weak};

pub enum AnalysisResult {
    AssertingClause(Rc<StoredClause>),
}

impl Solve<'_> {
    /// Either the most recent decision level in the resolution clause prior to the current level or 0.
    fn decision_level(&self, stored_clause: &Rc<StoredClause>) -> usize {
        let mut top_two = [None; 2];
        for lit in stored_clause.clause().literals() {
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
            _ => panic!("!"),
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
            _ => match self.analysis_switch(conflict_clause) {
                AnalysisResult::AssertingClause(asserting_clause) => {
                    let backjump_level = self.decision_level(&asserting_clause);

                    initialise_watches_for(
                        &asserting_clause,
                        &self.valuation_at(backjump_level),
                        &mut self.variables,
                    );

                    self.backjump(backjump_level);

                    Ok(SolveOk::AssertingClause)
                }
            },
        }
    }

    pub fn analysis_switch(&mut self, conflict_clause: Rc<StoredClause>) -> AnalysisResult {
        match self.config.analysis {
            3 => self.simple_analysis_three(conflict_clause),
            _ => panic!("Unknown analysis"),
        }
    }

    /// Common steps to storing a clause
    fn store_clause_common(&mut self, clause: ClauseVec, source: ClauseSource) -> Rc<StoredClause> {
        let stored_clause = self.store_clause(clause, source);
        stored_clause.set_lbd(&self.variables);
        stored_clause
    }

    /// Simple analysis performs resolution on any clause used to obtain a conflict literal at the current decision

    pub fn simple_analysis_three(&mut self, conflict_clause: Rc<StoredClause>) -> AnalysisResult {
        let mut resolved_clause = conflict_clause.clause().as_vec();
        let mut resolution_trail = vec![];

        let previous_level_val = self.valuation_at(self.current_level().index() - 1);

        for (src, _lit) in self.current_level().observations().iter().rev() {
            if resolved_clause.asserts(&previous_level_val).is_some() {
                break;
            }

            if let LiteralSource::StoredClause(cls) = src {
                if let Some(stored_clause) = cls.upgrade() {
                    let src_cls_vec = stored_clause.clause().as_vec();
                    let counterparts = find_counterpart_literals(&resolved_clause, &src_cls_vec);

                    if let Some(counterpart) = counterparts.first() {
                        resolution_trail.push(cls.clone());
                        resolved_clause =
                            resolve_sorted_clauses(&resolved_clause, &src_cls_vec, *counterpart)
                                .unwrap()
                                .to_vec()
                    }
                } else {
                    panic!("Clause has been dropped")
                }
            }
        }

        /*
        If some literals are known then their negation can be safely removed from the learnt clause.
        Though, this isn't a particular effective method…
         */
        if !self.top_level().observations().is_empty() {
            resolved_clause = resolved_clause
                .iter()
                .filter(|l| {
                    !self
                        .top_level()
                        .observations()
                        .iter()
                        .any(|(_, x)| l.negate() == *x)
                })
                .cloned()
                .collect::<Vec<_>>();
        }
        let stored_clause =
            self.store_clause_common(resolved_clause, ClauseSource::Resolution(resolution_trail));

        AnalysisResult::AssertingClause(stored_clause)
    }

    pub fn core(&self) {
        println!();
        println!("An unsatisfiable core:");
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
            println!("\t{}", clause.clause().as_string())
        }
        println!();
    }
}

pub fn extant_origins(clauses: Vec<Weak<StoredClause>>) -> Vec<Rc<StoredClause>> {
    let mut origin_nodes = BTreeSet::new();

    let mut q: VecDeque<Weak<StoredClause>> = VecDeque::new();
    for clause in clauses {
        q.push_back(clause);
    }
    loop {
        if q.is_empty() {
            break;
        }

        let node = q.pop_front().expect("Ah, the queue was empty…");
        if let Some(stored_clause) = node.upgrade() {
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
        } else {
            panic!("Lost clause")
        }
    }
    origin_nodes.into_iter().collect::<Vec<_>>()
}
