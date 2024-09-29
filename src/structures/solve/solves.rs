use crate::procedures::hobson_choices;
use crate::structures::solve::{Solve, SolveError, SolveOk, SolveStats};
use crate::structures::{ClauseStatus, Literal, LiteralSource, StoredClause, Valuation};
use std::rc::Rc;

pub enum SolveResult {
    Satisfiable,
    Unsatisfiable,
    Unknown,
}

macro_rules! propagation_macro {
    ($self:ident,
        $sc_vec:expr,
        $some_deduction:ident,
        $stats:ident,
        $conflict:ident
    ) => {
        for i in 0..$sc_vec.len() {
            let stored_clause = $sc_vec[i].clone();

            match stored_clause.watch_choices(&$self.valuation) {
                ClauseStatus::Entails(consequent) => {
                    let this_implication_time = std::time::Instant::now();
                    match $self.set_literal(
                        consequent,
                        LiteralSource::StoredClause(stored_clause.clone()),
                    ) {
                        Err(SolveError::Conflict(_, _)) => {
                            panic!("Conflict when setting a variable")
                        }
                        Err(e) => {
                            panic!("Unexpected error {e:?} when setting literal {consequent}")
                        }
                        Ok(()) => {
                            $some_deduction = true;
                        }
                    }
                    $stats.implication_time += this_implication_time.elapsed();
                }
                ClauseStatus::Conflict => {
                    match $conflict {
                        Conflicts::None => {
                            $conflict = Conflicts::Single(stored_clause);
                        }
                        Conflicts::Multiple(ref mut vec) => vec.push(stored_clause),
                        Conflicts::Single(_) => panic!("Conflict already set"),
                    };
                    match $self.config.break_on_first {
                        true => break,
                        false => continue,
                    };
                }
                ClauseStatus::Unsatisfied => (),
                ClauseStatus::Satisfied => (),
            }
        }
    };
}

#[derive(PartialEq)]
enum Conflicts {
    None,
    Single(Rc<StoredClause>),
    Multiple(Vec<Rc<StoredClause>>),
}

impl Solve<'_> {
    pub fn implication_solve(&mut self) -> (SolveResult, SolveStats) {
        let this_total_time = std::time::Instant::now();

        let mut stats = SolveStats::new();

        self.set_from_lists(hobson_choices(self.clauses())); // settle any literals which occur only as true or only as false

        let result: SolveResult;

        'main_loop: loop {
            stats.iterations += 1;

            log::trace!("Loop on valuation: {}", self.valuation.as_internal_string());

            let this_examination_time = std::time::Instant::now();

            stats.examination_time += this_examination_time.elapsed();

            let mut some_deduction = false;
            let mut conflicts = match self.config.break_on_first {
                true => Conflicts::None,
                false => Conflicts::Multiple(vec![]),
            };

            if self.current_level().get_choice().is_some() {
                let current_level = self.current_level().index();
                let literals = self.levels[current_level].updated_watches().clone();

                for literal in literals {
                    let v_id = literal.v_id;
                    propagation_macro!(
                        self,
                        self.variables[v_id].watch_occurrences(),
                        some_deduction,
                        stats,
                        conflicts
                    );
                    // TODO: Improve handling of multiple conflicts.
                    // If the conflict was due to an implication, then the implication is also a conflict…
                    match conflicts {
                        Conflicts::Single(_) | Conflicts::Multiple(_) => {
                            if self.config.break_on_first {
                                break;
                            }
                        }
                        _ => (),
                    }
                }
            } else {
                propagation_macro!(self, self.formula_clauses, some_deduction, stats, conflicts);

                propagation_macro!(self, self.learnt_clauses, some_deduction, stats, conflicts);
            }

            if conflicts != Conflicts::None {
                match conflicts {
                    Conflicts::None => (),
                    Conflicts::Single(stored_conflict) => {
                        match process_conflict_and_fix(self, &stored_conflict, &mut stats) {
                            false => {
                                result = SolveResult::Unsatisfiable;
                                break 'main_loop;
                            }
                            true => {
                                continue 'main_loop;
                            }
                        }
                    }
                    Conflicts::Multiple(conflict_vec) => {
                        if !conflict_vec.is_empty() {
                            match process_conflicts_and_fixes(self, conflict_vec, &mut stats) {
                                false => {
                                    result = SolveResult::Unsatisfiable;
                                    break 'main_loop;
                                }
                                true => (),
                            }
                        }
                    }
                }
            }

            if !some_deduction {
                if let Some(available_v_id) = self.most_active_none(&self.valuation) {
                    if self.time_to_reduce() {
                        reduce(self, &mut stats)
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
                    result = SolveResult::Satisfiable;
                    break 'main_loop;
                }
            }
        }

        stats.total_time = this_total_time.elapsed();
        match result {
            SolveResult::Satisfiable => {
                println!(
                    "c ASSIGNMENT: {}",
                    self.valuation.to_vec().as_display_string(self)
                );
            }
            SolveResult::Unsatisfiable => {}
            SolveResult::Unknown => {}
        }
        (result, stats)
    }
}

#[inline(always)]
fn reduce(solve: &mut Solve, stats: &mut SolveStats) {
    let this_reduction_time = std::time::Instant::now();
    println!("time to reduce");

    let learnt_count = solve.learnt_clauses.len();
    println!("Learnt count: {}", learnt_count);

    /*
    Clauses are removed from the learnt clause vector by swap_remove.
    So, when working through the vector it's importnat to only increment the pointer if no drop takes place.
     */
    let mut i = 0;
    loop {
        if i >= solve.learnt_clauses.len() {
            break;
        }
        let clause = solve.learnt_clauses[i].clone();
        if clause.lbd() > solve.config.min_glue_strength {
            solve.drop_clause_by_swap(&clause);
        } else {
            i += 1
        }
    }
    solve.forgets += 1;
    solve.conflcits_since_last_forget = 0;
    stats.reduction_time += this_reduction_time.elapsed();
    println!("Reduced to: {}", solve.learnt_clauses.len());
}

enum ProcessOption {
    Unsatisfiable,
    ContinueMain,
    Implicationed,
    Conflict,
}

fn process_clause(
    solve: &mut Solve,
    stored_clause: &Rc<StoredClause>,
    clause_status: &ClauseStatus,
    stats: &mut SolveStats,
    some_conflict: &mut bool,
    some_deduction: &mut bool,
) -> ProcessOption {
    match clause_status {
        ClauseStatus::Entails(consequent) => {
            let this_implication_time = std::time::Instant::now();
            match solve.set_literal(
                *consequent,
                LiteralSource::StoredClause(stored_clause.clone()),
            ) {
                Err(SolveError::Conflict(_, _)) => {
                    *some_conflict = true;
                }
                Err(e) => panic!("Unexpected error {e:?} when setting literal {consequent}"),
                Ok(()) => {
                    *some_deduction = true;
                }
            }
            stats.implication_time += this_implication_time.elapsed();
            ProcessOption::Implicationed
        }
        ClauseStatus::Conflict => ProcessOption::Conflict,
        _ => panic!("Something unexpected"),
    }
}

fn process_conflict_and_fix(
    solve: &mut Solve,
    stored_conflict: &Rc<StoredClause>,
    stats: &mut SolveStats,
) -> bool {
    let this_unsat_time = std::time::Instant::now();
    solve.notice_conflict(stored_conflict);
    stats.conflicts += 1;
    match solve.attempt_fix(stored_conflict.clone()) {
        Err(SolveError::NoSolution) => {
            if solve.config.core {
                solve.core();
            }
            false
        }
        Ok(SolveOk::AssertingClause) | Ok(SolveOk::Deduction(_)) => {
            stats.unsat_time += this_unsat_time.elapsed();
            true
        }
        Ok(ok) => panic!("Unexpected ok {ok:?} when attempting a fix"),
        Err(err) => {
            panic!("Unexpected error {err:?} when attempting a fix")
        }
    }
}

fn process_conflicts_and_fixes(
    solve: &mut Solve,
    stored_conflicts: Vec<Rc<StoredClause>>,
    stats: &mut SolveStats,
) -> bool {
    let this_unsat_time = std::time::Instant::now();
    for conflict in &stored_conflicts {
        solve.notice_conflict(conflict);
        stats.conflicts += 1;
    }

    match solve.attempt_fixes(stored_conflicts) {
        Err(SolveError::NoSolution) => {
            if solve.config.core {
                solve.core();
            }
            false
        }
        Ok(SolveOk::AssertingClause) | Ok(SolveOk::Deduction(_)) => {
            stats.unsat_time += this_unsat_time.elapsed();
            true
        }
        Ok(ok) => panic!("Unexpected ok {ok:?} when attempting a fix"),
        Err(err) => {
            panic!("Unexpected error {err:?} when attempting a fix")
        }
    }
}
