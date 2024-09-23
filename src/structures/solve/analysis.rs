use crate::procedures::binary_resolve_sorted_clauses;
use crate::structures::solve::{Solve, SolveError, SolveOk};
use crate::structures::{
    Clause, ClauseId, ClauseSource, ClauseVec, LiteralSource, StoredClause, Valuation,
};

use std::collections::BTreeSet;
use std::rc::Rc;

impl Solve<'_> {
    pub fn analyse_conflict(
        &mut self,
        stored_clause: Rc<StoredClause>,
    ) -> Result<SolveOk, SolveError> {
        // match self.simple_analysis_one(stored_clause) {
        match self.simple_analysis_two(stored_clause.clone()) {
            Some((antecedents, resolved_clause)) => {
                match resolved_clause.len() {
                    0 => panic!("Empty clause from analysis"),
                    1 => {
                        let the_literal = *resolved_clause.first().unwrap();
                        Ok(SolveOk::Deduction(the_literal))
                    }
                    _ => {
                        // the relevant backtrack level is either 0 is analysis is being performed at 0 or the first decision level in the resolution clause prior to the current level.
                        // For, if a prior level does *not* appear in the resolution clause then the level provided no relevant information.
                        let backjump_level = self
                            .decision_levels_of(&resolved_clause)
                            .filter(|level| *level != self.current_level().index())
                            .max()
                            .unwrap_or(0);

                        let expected_valuation = self.valuation_before_choice_at(backjump_level);

                        let id = self.add_clause(resolved_clause, ClauseSource::Resolution, &expected_valuation);
                        self.resolution_graph.add_resolution_by_ids(antecedents.iter().cloned(), id);

                        Ok(SolveOk::AssertingClause(backjump_level))
                    }
                }
            }
            None => panic!("Unexpected result from basic analysis"),
        }
    }

    pub fn attempt_fix(&mut self, stored_clause: Rc<StoredClause>) -> Result<SolveOk, SolveError> {
        let the_id = stored_clause.id();
        log::warn!(
            "Attempting fix on clause {the_id} at level {}",
            self.current_level().index()
        );
        match self.current_level().index() {
            0 => Err(SolveError::NoSolution),
            _ => match self.analyse_conflict(stored_clause) {
                Ok(SolveOk::AssertingClause(level)) => {
                    self.backjump(level);
                    Ok(SolveOk::AssertingClause(level))
                }
                Ok(SolveOk::Deduction(literal)) => {
                    self.backjump(0);
                    let _ = self.set_literal(literal, LiteralSource::Deduced);
                    Ok(SolveOk::Deduction(literal))
                }
                _ => panic!("Analysis failed given: Clause: {the_id}"),
            },
        }
    }

    /// Simple analysis performs resolution on any clause used to obtain a conflict literal at the current decision level.
    pub fn simple_analysis_one(&mut self, stored_clause: Rc<StoredClause>) -> Option<ClauseVec> {
        let mut the_resolved_clause = stored_clause.clause().as_vec();

        'resolution_loop: loop {
            log::trace!("Analysis clause: {}", the_resolved_clause.as_string());
            // the current choice will never be a resolution literal, as these are those literals in the clause which are the result of propagation
            let resolution_literals = self
                .implication_graph
                .resolution_candidates_at_level(&the_resolved_clause, self.current_level().index())
                .collect::<BTreeSet<_>>();

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

    pub fn simple_analysis_two(
        &mut self,
        stored_clause: Rc<StoredClause>,
    ) -> Option<(Vec<ClauseId>, ClauseVec)> {
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

        let mut clauses_used = vec![];

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
                            the_resolved_clause = binary_resolve_sorted_clauses(
                                &the_resolved_clause,
                                &path_clause.clause().as_vec(),
                                shared_literal.v_id,
                            )
                            .expect("Resolution failed")
                            .as_vec();
                            clauses_used.push(path_clause.id());
                        };
                    }
                }
            }
        }

        println!("Clauses used: {:?}", clauses_used);
        Some((clauses_used, the_resolved_clause))
    }
}
