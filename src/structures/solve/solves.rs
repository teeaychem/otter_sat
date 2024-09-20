use crate::procedures::hobson_choices;
use crate::structures::solve::{Solve, SolveError, SolveOk};
use crate::structures::{LiteralSource, Valuation, ValuationVec};

impl Solve<'_> {
    pub fn implication_solve(&mut self) -> Result<Option<ValuationVec>, SolveError> {
        println!("~~~ an implication solve ~~~");
        self.set_from_lists(hobson_choices(self.clauses())); // settle any literals which occur only as true or only as false

        'main_loop: loop {
            log::warn!("Loop on valuation: {}", self.valuation.as_internal_string());
            let status = self.examine_clauses_on(&self.valuation);

            if !status.choice_conflicts.is_empty() {
                let (clause_id, literal) = self.select_conflict(&status.choice_conflicts).unwrap();
                log::warn!("Selected a conflict");
                match self.attempt_fix(clause_id, Some(literal)) {
                    Err(SolveError::NoSolution) => {
                        return Ok(None);
                    }
                    Ok(SolveOk::AssertingClause(_)) | Ok(SolveOk::Deduction(_)) => {
                        continue 'main_loop;
                    }
                    Ok(ok) => panic!("Unexpected ok {ok:?} when attempting a fix"),
                    Err(err) => panic!("Unexpected error {err:?} when attempting a fix"),
                }
            }

            if !status.unsat.is_empty() {
                let (clause_id, literal) = self.select_unsat(&status.unsat).unwrap();
                log::warn!("Selected an unsatisfied clause");
                match self.attempt_fix(clause_id, Some(literal)) {
                    Err(SolveError::NoSolution) => {
                        return Ok(None);
                    }
                    Ok(SolveOk::AssertingClause(_)) | Ok(SolveOk::Deduction(_)) => {
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
                        Ok(SolveOk::AssertingClause(_)) | Ok(SolveOk::Deduction(_)) => {
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

                println!(
                    "\n\nChose {a_choice} @ {}\n\n",
                    self.current_level().index()
                );

                let _ = self.set_literal(*a_choice, LiteralSource::Choice);
                continue 'main_loop;
            }

            return Ok(Some(self.valuation.clone()));
        }
    }
}
