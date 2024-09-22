use crate::procedures::hobson_choices;
use crate::structures::solve::{Solve, SolveError, SolveOk};
use crate::structures::{Literal, LiteralSource, Valuation, ValuationVec};

impl Solve<'_> {
    pub fn implication_solve(&mut self) -> Result<Option<ValuationVec>, SolveError> {
        println!("~~~ an implication solve ~~~");
        self.set_from_lists(hobson_choices(self.clauses())); // settle any literals which occur only as true or only as false

        'main_loop: loop {
            log::warn!("Loop on valuation: {}", self.valuation.as_internal_string());
            let status = match self.current_level().get_choice() {
                None => self.examine_all_clauses_on(&self.valuation),
                Some(_) => self.examine_level_clauses_on(&self.valuation),
            };

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

            if let Some(available_v_id) = self.valuation.some_none() {
                // make a choice

                // log::warn!("Choice of {a_choice} @ {}\n", self.current_level().index());

                let _ = self.set_literal( Literal::new(available_v_id, true), LiteralSource::Choice);
                continue 'main_loop;
            }

            return Ok(Some(self.valuation.clone()));
        }
    }
}