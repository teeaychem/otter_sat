use crate::{
    clause,
    structures::{Literal, LiteralSource, Solve, SolveError, ValuationVec, VariableId},
    ClauseId, SolveOk, StoredClause, Valuation, ValuationError,
};
use std::collections::BTreeSet;

impl Solve<'_> {
    /// general order for pairs related to booleans is 0 is false, 1 is true
    pub fn hobson_choices(&self) -> (Vec<VariableId>, Vec<VariableId>) {
        let the_true: BTreeSet<VariableId> =
            self.literals_of_polarity(true).map(|l| l.v_id).collect();
        let the_false: BTreeSet<VariableId> =
            self.literals_of_polarity(false).map(|l| l.v_id).collect();
        let hobson_false: Vec<_> = the_false.difference(&the_true).cloned().collect();
        let hobson_true: Vec<_> = the_true.difference(&the_false).cloned().collect();
        (hobson_false, hobson_true)
    }

    pub fn settle_hobson_choices(&mut self) {
        let the_choices = self.hobson_choices();
        the_choices.0.iter().for_each(|&v_id| {
            let the_choice = Literal::new(v_id, false);
            let _ = self.set_literal(the_choice, LiteralSource::HobsonChoice);
        });
        the_choices.1.iter().for_each(|&v_id| {
            let the_choice = Literal::new(v_id, true);
            let _ = self.set_literal(the_choice, LiteralSource::HobsonChoice);
        });
    }

    pub fn attempt_fix(
        &mut self,
        clause_id: ClauseId,
        literal: Option<Literal>,
    ) -> Result<SolveOk, SolveError> {
        if self.current_level().index() == 0 {
            Err(SolveError::NoSolution)
        } else {
            let literal = literal.unwrap();
            let stored_clause = self.find_stored_clause(clause_id).expect("Missing clause");
            println!("Attempting fix given clause: {}", stored_clause.to_string());

            match self.analyse_conflict(clause_id, Some(literal)) {
                Ok(SolveOk::AssertingClause(level)) => {
                    log::warn!("Asserting clause at level {}", level);
                    while self.current_level().index() != 0 && self.current_level().index() >= level {
                        let _ = self.backtrack();
                    }
                    Ok(SolveOk::AssertingClause(level))
                }
                _ => panic!("Analysis failed"),
            }
        }
    }

    /*
    If a clause is unsatisfiable due to a valuation which conflicts with each literal of the clause, then at least one such conflicting literal was set at the current level.
    This function returns some clause and mentioned literal from a list of unsatisfiable clauses.
     */
    pub fn select_unsat(&self, clauses: &[ClauseId]) -> Option<(ClauseId, Literal)> {
        if !clauses.is_empty() {
            let the_clause_id = *clauses.first().unwrap();
            let the_stored_clause = self.find_stored_clause(the_clause_id).expect("oops");
            let current_variables = self.current_level().variables().collect::<BTreeSet<_>>();
            let mut overlap = the_stored_clause
                .clause
                .iter()
                .filter(|l| current_variables.contains(&l.v_id));
            let the_literal = *overlap.next().expect("No overlap");
            Some((the_clause_id, the_literal))
        } else {
            None
        }
    }

    pub fn select_conflict(&self, clauses: &[(ClauseId, Literal)]) -> Option<(ClauseId, Literal)> {
        if !clauses.is_empty() {
            Some(clauses.first().unwrap()).cloned()
        } else {
            None
        }
    }

    pub fn implication_solve(&mut self) -> Result<Option<ValuationVec>, SolveError> {
        println!("~~~ an implication solve ~~~");
        self.settle_hobson_choices(); // settle any literals which occur only as true or only as false

        'main_loop: loop {
            let status = self.examine_clauses_on(&self.valuation_at(self.current_level().index()));

            if !status.choice_conflicts.is_empty() {
                let (clause_id, literal) = self.select_conflict(&status.choice_conflicts).unwrap();
                match self.attempt_fix(clause_id, Some(literal)) {
                    Err(SolveError::NoSolution) => {
                        return Ok(None);
                    }
                    Ok(SolveOk::AssertingClause(_)) => {
                        continue 'main_loop;
                    }
                    Ok(ok) => panic!("Unexpected ok {ok:?} when attempting a fix"),
                    Err(err) => panic!("Unexpected error {err:?} when attempting a fix"),
                }
            }

            if !status.unsat.is_empty() {
                let (clause_id, literal) = self.select_unsat(&status.unsat).unwrap();
                match self.attempt_fix(clause_id, Some(literal)) {
                    Err(SolveError::NoSolution) => {
                        return Ok(None);
                    }
                    Ok(SolveOk::AssertingClause(_)) => {
                        continue 'main_loop;
                    }
                    Ok(ok) => panic!("Unexpected ok {ok:?} when attempting a fix"),
                    Err(err) => panic!("Unexpected error {err:?} when attempting a fix"),
                }
            }

            if !status.implications.is_empty() {
                let mut unsat_clauses = vec![];

                'implication: for (clause_id, consequent) in status.implications {
                    match self.set_literal(consequent, LiteralSource::StoredClause(clause_id)) {
                        Err(SolveError::Conflict(clause_id, literal)) => {
                            unsat_clauses.push((clause_id, literal));
                        }
                        Err(e) => panic!("Unexpected error {e:?} from setting a literal"),
                        Ok(()) => {
                            continue 'implication;
                        }
                    }
                }
                if let Some((clause, literal)) = self.select_conflict(&unsat_clauses) {
                    match self.attempt_fix(clause, Some(literal)) {
                        Err(SolveError::NoSolution) => {
                            return Ok(None);
                        }
                        Ok(SolveOk::AssertingClause(_)) => {
                            continue 'main_loop;
                        }
                        Ok(ok) => panic!("Unexpected ok {ok:?} when attempting a fix"),
                        Err(err) => panic!("Unexpected error {err:?} when attempting a fix"),
                    }
                }
                continue 'main_loop;
            }

            if !status.choices.is_empty() {
                // make a choice
                let a_choice = status.choices.first().unwrap();

                println!("\n\nChose {a_choice} @ {}\n\n", self.current_level().index());

                let _ = self.set_literal(*a_choice, LiteralSource::Choice);
                continue 'main_loop;
            }

            return Ok(Some(self.valuation.clone()));
        }
    }
}
