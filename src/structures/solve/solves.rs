use crate::procedures::hobson_choices;
use crate::structures::solve::{mutation::process_watches, Solve, SolveError, SolveOk, SolveStats};
use crate::structures::{
    ClauseStatus, Level, Literal, LiteralSource, StoredClause, Valuation,
    ValuationError, Variable,
};
use std::collections::VecDeque;
use std::mem;
use std::rc::Rc;

pub enum SolveResult {
    Satisfiable,
    Unsatisfiable,
    Unknown,
}

#[derive(PartialEq)]
enum Conflicts {
    No,
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

            let mut conflicts = match self.config.break_on_first {
                true => Conflicts::No,
                false => Conflicts::Multiple(vec![]),
            };

            'propagation_loop: while let Some(literal) = self.watch_q.pop_front() {
                let mut temprary_clause_vec: Vec<Rc<StoredClause>> = Vec::default();
                macro_rules! swap_occurrence_vecs {
                    /* perform a temporary swap of the relevant occurrence vector to allow mutable borrows of the solve variables when processing watch choices
                    the first swap takes place immediately, and the remaining swaps happen whenever the current iteration of the loop exits
                    the swap is safe, as the literal has been set already and will never be chosen as a watch
                     */
                    () => {
                        match literal.polarity {
                            false => mem::swap(
                                &mut self.variables[literal.v_id].positive_watch_occurrences,
                                &mut temprary_clause_vec,
                            ),
                            true => mem::swap(
                                &mut self.variables[literal.v_id].negative_watch_occurrences,
                                &mut temprary_clause_vec,
                            ),
                        };
                    };
                }
                swap_occurrence_vecs!();

                for stored_clause in &temprary_clause_vec {
                    match stored_clause.watch_choices(&self.valuation) {
                        ClauseStatus::Entails(consequent) => {
                            let this_implication_time = std::time::Instant::now();
                            literal_update(
                                consequent,
                                LiteralSource::StoredClause(stored_clause.clone()),
                                &mut self.levels,
                                &mut self.variables,
                                &mut self.valuation,
                                &mut self.watch_q,
                            );
                            stats.implication_time += this_implication_time.elapsed();
                        }
                        ClauseStatus::Conflict => {
                            match conflicts {
                                Conflicts::No => {
                                    conflicts = Conflicts::Single(stored_clause.clone());
                                }
                                Conflicts::Multiple(ref mut vec) => vec.push(stored_clause.clone()),
                                Conflicts::Single(_) => panic!("Conflict already set"),
                            };
                            match self.config.break_on_first {
                                true => {
                                    swap_occurrence_vecs!();
                                    if !temprary_clause_vec.is_empty() {
                                        println!("{}", temprary_clause_vec.len());
                                        panic!("wft {:?}", temprary_clause_vec);
                                    }
                                    break 'propagation_loop;
                                }
                                false => continue,
                            };
                        }
                        ClauseStatus::Unsatisfied => (),
                        ClauseStatus::Satisfied => (),
                    }
                }
                swap_occurrence_vecs!();
            }

            match conflicts {
                Conflicts::No => {
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
                        let _new_level = self.add_fresh_level();
                        let the_literal = Literal::new(available_v_id, false);

                        literal_update(
                            the_literal,
                            LiteralSource::Choice,
                            &mut self.levels,
                            &mut self.variables,
                            &mut self.valuation,
                            &mut self.watch_q,
                        );

                        stats.choice_time += this_choice_time.elapsed();

                        continue 'main_loop;
                    } else {
                        result = SolveResult::Satisfiable;
                        break 'main_loop;
                    }
                }
                Conflicts::Single(stored_conflict) => {
                    self.watch_q.clear();
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
                    self.watch_q.clear();
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
        // loop exit
        stats.total_time = this_total_time.elapsed();
        match result {
            SolveResult::Satisfiable => {
                if self.config.show_assignment {
                    println!(
                        "c ASSIGNMENT: {}",
                        self.valuation.to_vec().as_display_string(self)
                    );
                }
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

    let learnt_count = solve.learnt_clauses.len();
    log::warn!(target: "forget", "Learnt count: {}", learnt_count);

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
        if clause.lbd() > solve.config.glue_strength {
            solve.drop_clause_by_swap(&clause);
        } else {
            i += 1
        }
    }
    solve.forgets += 1;
    solve.conflcits_since_last_forget = 0;
    stats.reduction_time += this_reduction_time.elapsed();
    log::warn!(target: "forget", "Reduced to: {}", solve.learnt_clauses.len());
}

#[inline(always)]
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
        Err(err) => panic!("Unexpected {err:?} when attempting a fix"),
    }
}

#[inline(always)]
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
        Ok(ok) => panic!("Unexpected {ok:?} when attempting a fix"),
        Err(err) => {
            panic!("Unexpected {err:?} when attempting a fix")
        }
    }
}

#[inline(always)]
pub fn literal_update(
    literal: Literal,
    source: LiteralSource,
    levels: &mut [Level],
    variables: &mut [Variable],
    valuation: &mut impl Valuation,
    watch_q: &mut VecDeque<Literal>,
) {
    let variable = &mut variables[literal.v_id];

    // update the valuation and match the result
    match valuation.update_value(literal) {
        // if update occurrs, make records at the relevant level
        Ok(()) => {
            let level_index = match &source {
                LiteralSource::Choice | LiteralSource::StoredClause(_) => levels.len() - 1,
                LiteralSource::Assumption | LiteralSource::HobsonChoice => 0,
            };
            variable.set_decision_level(level_index);
            levels[level_index].record_literal(literal, &source);
            log::debug!("Set {source:?}: {literal}");

            // and, process whether any change to the watch literals is required, given an update has happened
            {
                let mut informative_literal = false;

                for sc in 0..variables[literal.v_id].positive_occurrences().len() {
                    let stored_clause = variables[literal.v_id].positive_occurrences()[sc].clone();
                    process_watches(
                        valuation,
                        variables,
                        &stored_clause,
                        literal,
                        &mut informative_literal,
                    );
                }
                for sc in 0..variables[literal.v_id].negative_occurrences().len() {
                    let stored_clause = variables[literal.v_id].negative_occurrences()[sc].clone();
                    process_watches(
                        valuation,
                        variables,
                        &stored_clause,
                        literal,
                        &mut informative_literal,
                    );
                }

                if informative_literal {
                    watch_q.push_back(literal);
                }
            }
        }
        Err(ValuationError::Match) => match source {
            LiteralSource::StoredClause(_) => {
                // A literal may be implied by multiple clauses, so there's no need to panic
                // rather, there's no need to do anything at all
            }
            _ => {
                log::error!("Attempt to restate {} via {:?}", literal, source);
                panic!("Attempt to restate {} via {:?}", literal, source)
            }
        },
        Err(ValuationError::Conflict) => {
            log::error!("Conflict when updating {literal} via {:?}", source);
            panic!("Conflict when updating {literal} via {:?}", source);
        }
    }
}
