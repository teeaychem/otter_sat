use crate::procedures::hobson_choices;
use crate::structures::{
    clause::{
        stored_clause::{ClauseStatus, StoredClause, WatchStatus, WatchUpdateEnum},
        Clause,
    },
    level::Level,
    literal::{Literal, LiteralSource},
    solve::{
        config::{
            config_glue_strength, config_hobson, config_restarts_allowed, config_show_assignment,
            config_show_core, config_show_stats, config_time_limit,
        },
        stats::SolveStats,
        Solve, {SolveResult, SolveStatus},
    },
    valuation::{Valuation, ValuationStatus},
    variable::Variable,
};
use std::rc::Rc;

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

        if config_hobson() {
            self.set_from_lists(hobson_choices(self.clauses()));
        }

        let result: SolveResult;

        'main_loop: loop {
            stats.total_time = this_total_time.elapsed();
            if config_time_limit().is_some_and(|t| stats.total_time > t) {
                if config_show_stats() {
                    println!("c TIME LIMIT EXCEEDED");
                }
                result = SolveResult::Unknown;
                break 'main_loop;
            }

            stats.iterations += 1;

            let mut conflicts = match crate::CONFIG_BREAK_ON_FIRST {
                true => Conflicts::No,
                false => Conflicts::Multiple(vec![]),
            };

            let this_implication_time = std::time::Instant::now();
            'propagation_loop: while let Some(literal) = self.watch_q.pop_front() {
                let mut temprary_clause_vec: Vec<Rc<StoredClause>> = Vec::default();
                macro_rules! swap_occurrence_vecs {
                    /*
                    perform a temporary swap of the relevant occurrence vector to allow mutable borrows of the solve variables when processing watch choices
                    the first swap takes place immediately, and the remaining swaps happen whenever the current iteration of the loop exits
                    the swap is safe, as the literal has been set already and will never be chosen as a watch
                     */
                    () => {
                        match literal.polarity {
                            false => std::mem::swap(
                                &mut self.variables[literal.v_id].positive_watch_occurrences,
                                &mut temprary_clause_vec,
                            ),
                            true => std::mem::swap(
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
                            literal_update(
                                consequent,
                                LiteralSource::StoredClause(stored_clause.clone()),
                                &mut self.levels,
                                &mut self.variables,
                                &mut self.valuation,
                            );
                            self.watch_q.push_back(consequent);
                        }
                        ClauseStatus::Conflict => {
                            match conflicts {
                                Conflicts::No => {
                                    conflicts = Conflicts::Single(stored_clause.clone());
                                }
                                Conflicts::Multiple(ref mut vec) => vec.push(stored_clause.clone()),
                                Conflicts::Single(_) => panic!("Conflict already set"),
                            };
                            match crate::CONFIG_BREAK_ON_FIRST {
                                true => {
                                    swap_occurrence_vecs!();
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
            stats.implication_time += this_implication_time.elapsed();

            match conflicts {
                Conflicts::No => {
                    if let Some(available_v_id) = self.most_active_none(&self.valuation) {
                        if self.is_it_time_to_reduce() {
                            log::debug!(target: "forget", "{stats}");
                            let this_reduction_time = std::time::Instant::now();
                            reduce(self);
                            if config_restarts_allowed() {
                                self.watch_q.clear();
                                self.backjump(0);
                            }

                            stats.reduction_time += this_reduction_time.elapsed();
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
                        );
                        self.watch_q.push_back(the_literal);

                        stats.choice_time += this_choice_time.elapsed();

                        continue 'main_loop;
                    } else {
                        result = SolveResult::Satisfiable;
                        break 'main_loop;
                    }
                }
                Conflicts::Single(stored_conflict) => {
                    self.watch_q.clear();
                    let this_unsat_time = std::time::Instant::now();
                    self.notice_conflict(&stored_conflict);
                    let analysis_result = self.attempt_fix(stored_conflict);
                    stats.unsat_time += this_unsat_time.elapsed();
                    match analysis_result {
                        SolveStatus::NoSolution => {
                            result = SolveResult::Unsatisfiable;
                            break 'main_loop;
                        }
                        SolveStatus::AssertingClause | SolveStatus::Deduction(_) => {
                            stats.conflicts += 1;
                            continue 'main_loop;
                        }
                        other => panic!("Unexpected {other:?} when attempting a fix"),
                    }
                }
                Conflicts::Multiple(conflict_vec) => {
                    self.watch_q.clear();
                    let this_unsat_time = std::time::Instant::now();
                    for conflict in &conflict_vec {
                        self.notice_conflict(conflict);
                        stats.conflicts += 1;
                    }
                    match self.attempt_fixes(conflict_vec) {
                        SolveStatus::NoSolution => {
                            result = SolveResult::Unsatisfiable;
                            break 'main_loop;
                        }
                        SolveStatus::AssertingClause | SolveStatus::Deduction(_) => {}
                        other => panic!("Unexpected {other:?} when attempting a fix"),
                    }
                    stats.unsat_time += this_unsat_time.elapsed();
                }
            }
        }
        // loop exit
        stats.total_time = this_total_time.elapsed();
        match result {
            SolveResult::Satisfiable => {
                if config_show_assignment() {
                    println!(
                        "c ASSIGNMENT: {}",
                        self.valuation.to_vec().as_display_string(self)
                    );
                }
            }
            SolveResult::Unsatisfiable => {
                if config_show_core() {
                    self.core();
                }
            }
            SolveResult::Unknown => {}
        }
        (result, stats)
    }
}

#[inline(always)]
fn reduce(solve: &mut Solve) {
    log::debug!(target: "forget", "Learnt count: {}", solve.learnt_clauses.len());

    {
        // solve.learnt_clauses.truncate(learnt_count / 2);
        let mut i = 0;
        let mut length = solve.learnt_clauses.len();
        while i < length {
            if solve.learnt_clauses[i].lbd() > config_glue_strength() {
                solve.drop_learnt_clause_by_swap(i);
                length -= 1;
            } else {
                i += 1
            }
        }
    }
    solve.forgets += 1;
    solve.conflcits_since_last_forget = 0;
    log::debug!(target: "forget", "Reduced to: {}", solve.learnt_clauses.len());
}

#[inline(always)]
pub fn literal_update(
    literal: Literal,
    source: LiteralSource,
    levels: &mut [Level],
    vars: &mut [Variable],
    valuation: &mut impl Valuation,
) {
    let variable = &mut vars[literal.v_id];

    // update the valuation and match the result
    match valuation.update_value(literal) {
        Ok(()) => {
            log::trace!("Set {source:?}: {literal}");
            // if update occurrs, make records at the relevant level
            let level_index = match &source {
                LiteralSource::Choice | LiteralSource::StoredClause(_) => levels.len() - 1,
                LiteralSource::Assumption | LiteralSource::HobsonChoice => 0,
            };
            variable.set_decision_level(level_index);
            levels[level_index].record_literal(literal, &source);

            // and, process whether any change to the watch literals is required

            // TODO: this is slower than duplicating the following loop for both +/- occurrence vecs
            // Though, as a function is not viable given the use of variables in the process functions,
            // this while unstable this allows updating code in only one place
            let mut working_clause_vec = match literal.polarity {
                true => std::mem::take(&mut vars[literal.v_id].negative_watch_occurrences),
                false => std::mem::take(&mut vars[literal.v_id].positive_watch_occurrences),
            };

            let mut index = 0;
            let mut length = working_clause_vec.len();
            while index < length {
                let stored_clause = &working_clause_vec[index];

                let process_update = match stored_clause.watched_a().v_id == literal.v_id {
                    true => process_watches(valuation, vars, stored_clause, Watch::A),
                    false => process_watches(valuation, vars, stored_clause, Watch::B),
                };

                match process_update {
                    WatchStatus::AlreadySatisfied
                    | WatchStatus::AlreadyImplication
                    | WatchStatus::AlreadyConflict => {
                        index += 1;
                    }
                    WatchStatus::NewImplication
                    | WatchStatus::NewSatisfied
                    | WatchStatus::NewTwoNone => {
                        working_clause_vec.swap_remove(index);
                        length -= 1;
                    }
                };
            }

            match literal.polarity {
                true => std::mem::replace(
                    &mut vars[literal.v_id].negative_watch_occurrences,
                    working_clause_vec,
                ),
                false => std::mem::replace(
                    &mut vars[literal.v_id].positive_watch_occurrences,
                    working_clause_vec,
                ),
            };
        }
        Err(ValuationStatus::Match) => match source {
            LiteralSource::StoredClause(_) => {
                // A literal may be implied by multiple clauses, so there's no need to do anything
            }
            _ => panic!("Attempt to restate {} via {:?}", literal, source),
        },
        Err(ValuationStatus::Conflict) => panic!("Conflict when update {literal} via {:?}", source),
        Err(_) => todo!(),
    }
}

#[derive(Debug, Clone)]
pub enum Watch {
    A,
    B,
}

pub fn process_watches(
    val: &impl Valuation,
    variables: &mut [Variable],
    stored_clause: &Rc<StoredClause>,
    chosen_watch: Watch,
) -> WatchStatus {
    match stored_clause.length() {
        1 => match val.of_v_id(stored_clause.clause[stored_clause.watch_a.get()].v_id) {
            None => WatchStatus::AlreadyImplication,
            Some(_) => WatchStatus::AlreadySatisfied,
        },
        _ => {
            macro_rules! update_the_watch_to {
                ($a:expr) => {
                    match chosen_watch {
                        Watch::A => {
                            stored_clause.update_watch_a($a);
                            let watched_a = stored_clause.watched_a();
                            variables[watched_a.v_id].watch_added(stored_clause, watched_a.polarity)
                        }
                        Watch::B => {
                            stored_clause.update_watch_b($a);
                            let watched_b = stored_clause.watched_b();
                            variables[watched_b.v_id].watch_added(stored_clause, watched_b.polarity)
                        }
                    }
                };
            }

            let watched_x_literal = match chosen_watch {
                Watch::A => stored_clause.clause[stored_clause.watch_a.get()],
                Watch::B => stored_clause.clause[stored_clause.watch_b.get()],
            };

            let watched_y_literal = match chosen_watch {
                Watch::A => stored_clause.clause[stored_clause.watch_b.get()],
                Watch::B => stored_clause.clause[stored_clause.watch_a.get()],
            };

            let watched_y_value = val.of_v_id(watched_y_literal.v_id);

            // the match below is ordered to avoid this comparison when possible
            // and the macro ensures it's only calculated when required
            macro_rules! watched_y_match {
                () => {
                    watched_y_value.is_some_and(|p| p == watched_y_literal.polarity)
                };
            }

            if let Some(_current_x_value) = val.of_v_id(watched_x_literal.v_id) {
                // if _current_a_value == watched_a_literal.polarity {
                //     panic!("watch already sat on watched")
                // }

                match stored_clause.some_none_or_else_witness_idx(val, watched_y_literal.v_id) {
                    WatchUpdateEnum::Witness(idx) => {
                        if watched_y_match!() {
                            WatchStatus::AlreadySatisfied
                        } else {
                            update_the_watch_to!(idx);
                            WatchStatus::NewSatisfied
                        }
                    }
                    WatchUpdateEnum::None(idx) => {
                        update_the_watch_to!(idx);
                        if watched_y_value.is_none() {
                            WatchStatus::NewTwoNone
                        } else if watched_y_match!() {
                            WatchStatus::NewSatisfied
                        } else {
                            WatchStatus::NewImplication
                        }
                    }
                    WatchUpdateEnum::No => {
                        if watched_y_value.is_none() {
                            WatchStatus::AlreadyImplication
                        } else if watched_y_match!() {
                            WatchStatus::AlreadySatisfied
                        } else {
                            WatchStatus::AlreadyConflict
                        }
                    }
                }
            } else {
                panic!("Process watches without value");
            }
        }
    }
}
