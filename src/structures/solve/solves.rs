use crate::procedures::hobson_choices;
use crate::structures::solve::{Solve, SolveError, SolveOk};
use crate::structures::{Clause, Literal, LiteralSource, Valuation};

pub enum SolveResult {
    Satisfiable,
    Unsatisfiable,
    Unknown,
}

pub struct SolveStats {
    pub total_time: std::time::Duration,
    pub examination_time: std::time::Duration,
    pub implication_time: std::time::Duration,
    pub unsat_time: std::time::Duration,
    pub reduction_time: std::time::Duration,
    pub choice_time: std::time::Duration,
    pub iterations: usize,
}

impl SolveStats {
    pub fn new() -> Self {
        SolveStats {
            total_time: std::time::Duration::new(0, 0),
            examination_time: std::time::Duration::new(0, 0),
            implication_time: std::time::Duration::new(0, 0),
            unsat_time: std::time::Duration::new(0, 0),
            reduction_time: std::time::Duration::new(0, 0),
            choice_time: std::time::Duration::new(0, 0),
            iterations: 0
        }
    }
}

impl std::fmt::Display for SolveStats {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        writeln!(f, "c STATS")?;
        writeln!(f, "c ITERATIONS: {}", self.iterations)?;
        writeln!(f, "c TIME: {:.2?}", self.total_time)?;
        writeln!(f, "c \tEXAMINATION: {:.2?}", self.examination_time)?;
        writeln!(f, "c \tIMPLICATION: {:.2?}", self.implication_time)?;
        writeln!(f, "c \tUNSAT: {:.2?}", self.unsat_time)?;
        writeln!(f, "c \tREDUCTION: {:.2?}", self.reduction_time)?;
        writeln!(f, "c \tCHOICE: {:.2?}", self.choice_time)?;
        Ok(())
    }
}

impl Solve<'_> {
    pub fn implication_solve(&mut self) -> (SolveResult, SolveStats) {
        println!("~~~ an implication solve ~~~");
        let this_total_time = std::time::Instant::now();

        let mut stats = SolveStats::new();

        self.set_from_lists(hobson_choices(self.clauses())); // settle any literals which occur only as true or only as false

        'main_loop: loop {
            stats.iterations += 1;
            log::trace!("Loop on valuation: {}", self.valuation.as_internal_string());

            let this_examination_time = std::time::Instant::now();
            let status = match self.current_level().get_choice() {
                None => self.examine_all_clauses_on(&self.valuation),
                Some(_) => self.examine_level_clauses_on(&self.valuation),
            };
            stats.examination_time += this_examination_time.elapsed();

            let mut unsat_clauses = status.unsat;
            let mut some_deduction = false;

            if unsat_clauses.is_empty() {
                let this_implication_time = std::time::Instant::now();
                'implication: for (stored_clause, consequent) in &status.implications {
                    match self.set_literal(
                        *consequent,
                        LiteralSource::StoredClause(stored_clause.clone()),
                    ) {
                        Err(SolveError::Conflict(stored_clause, _literal)) => {
                            unsat_clauses.push(stored_clause);
                        }
                        Err(e) => panic!("Unexpected error {e:?} from setting a literal"),
                        Ok(()) => {
                            if !some_deduction {
                                some_deduction = true
                            };
                            let length = stored_clause.length();
                            if length == 1 {
                                self.drop_clause(stored_clause);
                            }
                            continue 'implication;
                        }
                    }
                }
                stats.implication_time += this_implication_time.elapsed();
            }

            if let Some(stored_clause) = self.select_unsat(&unsat_clauses) {
                let this_unsat_time = std::time::Instant::now();
                self.process_unsat(&unsat_clauses);

                log::trace!("Selected an unsatisfied clause");
                match self.attempt_fix(stored_clause) {
                    Err(SolveError::NoSolution) => {
                        if self.config.core {
                            self.core();
                        }
                        stats.total_time = this_total_time.elapsed();
                        return (SolveResult::Unsatisfiable, stats);
                    }
                    Ok(SolveOk::AssertingClause) | Ok(SolveOk::Deduction(_)) => {
                        stats.unsat_time += this_unsat_time.elapsed();
                        continue 'main_loop;
                    }
                    Ok(ok) => panic!("Unexpected ok {ok:?} when attempting a fix"),
                    Err(err) => panic!("Unexpected error {err:?} when attempting a fix"),
                }
            }

            if !some_deduction {
                if let Some(available_v_id) = self.most_active_none(&self.valuation) {
                    if self.time_to_reduce() {
                        let this_reduction_time = std::time::Instant::now();
                        println!("time to reduce");
                        self.learnt_clauses.sort_unstable_by_key(|a| a.lbd());

                        let learnt_count = self.learnt_clauses.len();
                        println!("Learnt count: {}", learnt_count);
                        for _ in 0..learnt_count {
                            if self
                                .learnt_clauses
                                .last()
                                .is_some_and(|lc| lc.lbd() > self.config.min_glue_strength)
                            {
                                let goodbye = self.learnt_clauses.last().unwrap().clone();
                                self.drop_clause(&goodbye);
                            } else {
                                break;
                            }
                        }
                        self.forgets += 1;
                        self.conflcits_since_last_forget = 0;
                        stats.reduction_time += this_reduction_time.elapsed();
                        println!("Reduced to: {}", self.learnt_clauses.len());
                    }

                    let this_choice_time = std::time::Instant::now();
                    log::trace!(
                        "Choice: {available_v_id} @ {} with activity {}",
                        self.current_level().index(),
                        self.variables[available_v_id].activity()
                    );

                    let _ = self
                        .set_literal(Literal::new(available_v_id, false), LiteralSource::Choice);
                    stats.choice_time += this_choice_time.elapsed();

                    continue 'main_loop;
                } else {
                    println!(
                        "c ASSIGNMENT: {}",
                        self.valuation.to_vec().as_display_string(self)
                    );
                    stats.total_time = this_total_time.elapsed();
                    return (SolveResult::Satisfiable, stats);
                }
            }
        }
    }
}
