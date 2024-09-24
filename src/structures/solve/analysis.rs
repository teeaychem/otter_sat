use crate::procedures::binary_resolve_sorted_clauses;
use crate::structures::solve::{Solve, SolveError, SolveOk};
use crate::structures::{Clause, ClauseSource, ClauseVec, LiteralSource, StoredClause, Valuation};

use std::collections::BTreeSet;
use std::rc::Rc;

pub enum AnalysisResult {
    AssertingClause(Rc<StoredClause>),
}

impl Solve<'_> {
    // the relevant backtrack level is either 0 is analysis is being performed at 0 or the first decision level in the resolution clause prior to the current level.
    // For, if a prior level does *not* appear in the resolution clause then the level provided no relevant information.
    fn backjump_level(&self, stored_clause: Rc<StoredClause>) -> usize {
        self.decision_levels_of(stored_clause.clause())
            .filter(|level| *level != self.current_level().index())
            .max()
            .unwrap_or(0)
    }

    pub fn attempt_fix(
        &mut self,
        conflict_clause: Rc<StoredClause>,
    ) -> Result<SolveOk, SolveError> {
        let the_id = conflict_clause.id();
        log::warn!(
            "Attempting fix on clause {the_id} at level {}",
            self.current_level().index()
        );
        match self.current_level().index() {
            0 => Err(SolveError::NoSolution),
            _ => match self.simple_analysis_two(conflict_clause) {
                AnalysisResult::AssertingClause(asserting_clause) => {
                    let backjump_level = self.backjump_level(asserting_clause.clone());
                    let expected_valuation = self.valuation_before_choice_at(backjump_level);
                    asserting_clause.initialise_watches_for(&expected_valuation);

                    self.backjump(backjump_level);

                    Ok(SolveOk::AssertingClause)
                }
            },
        }
    }

    /// Simple analysis performs resolution on any clause used to obtain a conflict literal at the current decision level.
    pub fn simple_analysis_one(&mut self, stored_clause: Rc<StoredClause>) -> Option<ClauseVec> {
        let mut the_resolved_clause = stored_clause.clause().as_vec();

        'resolution_loop: loop {
            log::trace!("Analysis clause: {}", the_resolved_clause.as_string());
            // the current choice will never be a resolution literal, as these are those literals in the clause which are the result of propagation
            let mut resolution_literals = self
                .implication_graph
                .resolution_candidates_at_level(&the_resolved_clause, self.current_level().index())
                .collect::<Vec<_>>();
            resolution_literals.sort_unstable();
            resolution_literals.dedup();

            match resolution_literals.is_empty() {
                true => {
                    return Some(the_resolved_clause);
                }
                false => {
                    let (stored_clause, resolution_literal) =
                        resolution_literals.first().expect("No resolution literal");

                    the_resolved_clause = binary_resolve_sorted_clauses(
                        &the_resolved_clause.to_vec(),
                        &stored_clause.clause().as_vec(),
                        resolution_literal.v_id,
                    )
                    .expect("Resolution failed")
                    .as_vec();

                    continue 'resolution_loop;
                }
            }
        }
    }

    pub fn simple_analysis_two(&mut self, stored_clause: Rc<StoredClause>) -> AnalysisResult {
        log::warn!("Simple analysis two");
        log::warn!("The valuation is: {}", self.valuation.as_internal_string());
        /*
        Unsafe for the moment.

        At issue is temporarily updating the implication graph to include the conflict clause implying falsum and then examining the conflcit clause.
        For, ideally a conflict clause is only borrowed from the store of clauses, and this means either retreiving for the stored twice, or dereferencing the borrow so it can be used while mutably borrowing the solve to update the graph.
        As retreiving the stored clause is a basic find operation, unsafely dereferencing the borrow is preferred.
         */

        let the_conflict_clause = stored_clause;
        log::warn!(
            "Simple analysis two on: {}",
            the_conflict_clause.clause().as_string()
        );

        let mut the_resolved_clause = the_conflict_clause.clause().as_vec();
        let the_conflict_level_choice = {
            let conflict_decision_level = self
                .decision_levels_of(the_conflict_clause.clause())
                .max()
                .expect("No clause decision level");
            self.level_choice(conflict_decision_level)
        };

        let the_immediate_domiator = self
            .implication_graph
            .immediate_dominators(the_resolved_clause.literals(), the_conflict_level_choice)
            .expect("No immediate dominator");

        log::warn!("Resolution on paths…");

        let mut resolution_history = vec![];

        for literal in the_conflict_clause.literals() {
            match self
                .implication_graph
                .some_clause_path_between(the_immediate_domiator, literal.negate())
            {
                None => continue,
                Some(mut path_clauses) => {
                    path_clauses.reverse(); // Not strictly necessary
                    for path_clause in path_clauses {
                        if let Some(shared_literal) =
                            path_clause.clause().literals().find(|path_literal| {
                                the_resolved_clause.contains(&path_literal.negate())
                            })
                        {
                            resolution_history.push(path_clause.clone());
                            the_resolved_clause = binary_resolve_sorted_clauses(
                                &the_resolved_clause,
                                &path_clause.clause().as_vec(),
                                shared_literal.v_id,
                            )
                            .expect("Resolution failed")
                            .as_vec();
                        };
                    }
                }
            }
        }

        let sc = self.store_clause(the_resolved_clause, ClauseSource::Resolution);
        self.resolution_graph
            .add_resolution(resolution_history.iter().cloned(), sc.clone());

        AnalysisResult::AssertingClause(sc)
    }
}
