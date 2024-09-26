use crate::procedures::hobson_choices;
use crate::structures::solve::{Solve, SolveError, SolveOk};
use crate::structures::{Clause, Literal, LiteralSource, Valuation};
use std::rc::Rc;

pub enum SolveResult {
    Satisfiable,
    Unsatisfiable,
    Unkown,
}

impl Solve<'_> {
    pub fn implication_solve(&mut self) -> Result<SolveResult, SolveError> {
        println!("~~~ an implication solve ~~~");
        self.set_from_lists(hobson_choices(self.clauses())); // settle any literals which occur only as true or only as false

        'main_loop: loop {
            log::trace!("Loop on valuation: {}", self.valuation.as_internal_string());

            let status = match self.current_level().get_choice() {
                None => self.examine_all_clauses_on(&self.valuation),
                Some(_) => self.examine_level_clauses_on(&self.valuation),
            };

            let mut unsat_clauses = status.unsat;
            let mut some_deduction = false;

            if unsat_clauses.is_empty() {
                'implication: for (stored_clause, consequent) in &status.implications {
                    match self.set_literal(
                        *consequent,
                        LiteralSource::StoredClause(Rc::downgrade(stored_clause)),
                    ) {
                        Err(SolveError::Conflict(weak_clause, _literal)) => {
                            if let Some(stored_clause) = weak_clause.upgrade() {
                                unsat_clauses.push(stored_clause);
                            } else {
                                panic!("Lost conflict clause");
                            }
                        }
                        Err(e) => panic!("Unexpected error {e:?} from setting a literal"),
                        Ok(()) => {
                            if !some_deduction {
                                some_deduction = true
                            };
                            let length = stored_clause.clause().len();
                            if length == 1 {
                                self.drop_clause(stored_clause);
                            }
                            continue 'implication;
                        }
                    }
                }
            }

            if let Some(stored_clause) = self.select_unsat(&unsat_clauses) {
                self.process_unsat(&unsat_clauses);

                log::warn!("Selected an unsatisfied clause");
                match self.attempt_fix(stored_clause) {
                    Err(SolveError::NoSolution) => {
                        if self.config.core {
                            self.core();
                        }
                        return Ok(SolveResult::Unsatisfiable);
                    }
                    Ok(SolveOk::AssertingClause) | Ok(SolveOk::Deduction(_)) => {
                        continue 'main_loop;
                    }
                    Ok(ok) => panic!("Unexpected ok {ok:?} when attempting a fix"),
                    Err(err) => panic!("Unexpected error {err:?} when attempting a fix"),
                }
            }

            if !some_deduction {
                if let Some(available_v_id) = self.most_active_none(&self.valuation) {
                    if self.time_to_reduce() {
                        println!("time to reduce");
                        self.learnt_clauses.sort_unstable_by_key(|a| a.lbd());

                        let learnt_count = self.learnt_clauses.len();
                        for _ in 0..learnt_count / 2 {
                            if self.learnt_clauses.last().is_some_and(|lc| lc.lbd() > 2) {
                                let goodbye = self.learnt_clauses.last().unwrap().clone();
                                self.drop_clause(&goodbye);
                            } else {
                                break;
                            }
                        }
                        // self.learnt_clauses.truncate(learnt_count / 2);
                    }

                    log::trace!(
                        "Choice: {available_v_id} @ {} with activity {}",
                        self.current_level().index(),
                        self.variables[available_v_id].activity()
                    );

                    let _ = self
                        .set_literal(Literal::new(available_v_id, false), LiteralSource::Choice);

                    continue 'main_loop;
                } else {
                    println!(
                        "c ASSIGNMENT: {}",
                        self.valuation.to_vec().as_display_string(self)
                    );
                    return Ok(SolveResult::Satisfiable);
                }
            }
        }
    }
}
