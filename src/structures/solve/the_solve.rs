use crate::procedures::hobson_choices;
use crate::structures::{
    clause::stored_clause::{ClauseStatus, StoredClause, WatchStatus},
    level::Level,
    literal::{Literal, LiteralSource},
    solve::{
        config::{
            config_exploration_priority, config_glue_strength, config_show_assignment,
            config_show_core, ExplorationPriority,
        },
        core::{process_watches_two_a, process_watches_two_b},
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
        let exploration_priority = config_exploration_priority();

        let mut stats = SolveStats::new();

        self.set_from_lists(hobson_choices(self.clauses())); // settle any literals which occur only as true or only as false

        let result: SolveResult;

        'main_loop: loop {
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
                            match literal_update(
                                consequent,
                                LiteralSource::StoredClause(stored_clause.clone()),
                                &mut self.levels,
                                &mut self.variables,
                                &mut self.valuation,
                            ) {
                                WatchStatus::NewImplication | WatchStatus::AlreadyImplication => {
                                    match exploration_priority {
                                        ExplorationPriority::Implication => {
                                            self.watch_q.push_front(consequent)
                                        }
                                        _ => self.watch_q.push_back(consequent),
                                    }
                                }

                                WatchStatus::AlreadyConflict => match exploration_priority {
                                    ExplorationPriority::Conflict => {
                                        self.watch_q.push_front(consequent)
                                    }
                                    _ => self.watch_q.push_back(consequent),
                                },
                                _ => {}
                            }
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

                        match literal_update(
                            the_literal,
                            LiteralSource::Choice,
                            &mut self.levels,
                            &mut self.variables,
                            &mut self.valuation,
                        ) {
                            WatchStatus::NewImplication | WatchStatus::AlreadyImplication => {
                                match exploration_priority {
                                    ExplorationPriority::Implication => {
                                        self.watch_q.push_front(the_literal)
                                    }
                                    _ => self.watch_q.push_back(the_literal),
                                }
                            }

                            WatchStatus::AlreadyConflict => match exploration_priority {
                                ExplorationPriority::Conflict => {
                                    self.watch_q.push_front(the_literal)
                                }
                                _ => self.watch_q.push_back(the_literal),
                            },
                            _ => {}
                        };

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
                    let analysis_result = self.attempt_fix(stored_conflict.clone());
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
    let learnt_count = solve.learnt_clauses.len();
    log::debug!(target: "forget", "Learnt count: {}", learnt_count);

    /*
    Clauses are removed from the learnt clause vector by swap_remove.
    So, when working through the vector it's importnat to only increment the pointer if no drop takes place.
     */
    {
        let mut i = 0;
        let limit = learnt_count / 2;
        loop {
            if i >= solve.learnt_clauses.len() || i > limit {
                break;
            }
            let clause = solve.learnt_clauses[i].clone();

            if clause.lbd() > config_glue_strength() {
                solve.drop_clause_by_swap(&clause);
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
    variables: &mut [Variable],
    valuation: &mut impl Valuation,
) -> WatchStatus {
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
            log::trace!("Set {source:?}: {literal}");

            // and, process whether any change to the watch literals is required, given an update has happened
            {
                let mut watch_status = WatchStatus::TwoNone;
                // note: when using simple watch update inspect all possible occurrences

                // TODO: this is slower than duplicating the following loop for both +/- occurrence vecs
                // Though, as a function is not viable given the use of variables in the process functions,
                // this while unstable this allows updating code in only one place
                let mut working_clause_vec = match literal.polarity {
                    true => std::mem::take(&mut variables[literal.v_id].negative_watch_occurrences),
                    false => {
                        std::mem::take(&mut variables[literal.v_id].positive_watch_occurrences)
                    }
                };

                let mut index = 0;
                while index < working_clause_vec.len() {
                    let stored_clause = working_clause_vec[index].clone();

                    let process_update = if stored_clause.watched_a().v_id == literal.v_id {
                        process_watches_two_a(valuation, variables, &stored_clause)
                    } else {
                        process_watches_two_b(valuation, variables, &stored_clause)
                    };

                    match process_update {
                        WatchStatus::AlreadySatisfied => {
                            index += 1;
                            if watch_status != WatchStatus::AlreadyConflict {
                                watch_status = WatchStatus::AlreadySatisfied
                            }
                        }
                        WatchStatus::AlreadyImplication => {
                            index += 1;
                            if watch_status != WatchStatus::AlreadyConflict {
                                watch_status = WatchStatus::AlreadyImplication
                            }
                        }
                        WatchStatus::AlreadyConflict => {
                            watch_status = WatchStatus::AlreadyConflict;
                            index += 1;
                        }
                        WatchStatus::NewImplication => {
                            working_clause_vec.swap_remove(index);
                            if watch_status != WatchStatus::AlreadyConflict {
                                watch_status = WatchStatus::NewImplication
                            }
                        }
                        WatchStatus::NewSatisfied => {
                            working_clause_vec.swap_remove(index);
                            if watch_status != WatchStatus::AlreadyConflict {
                                watch_status = WatchStatus::NewSatisfied
                            }
                        }
                        WatchStatus::TwoNone => {
                            working_clause_vec.swap_remove(index);
                        }
                    };
                }

                match literal.polarity {
                    true => std::mem::replace(
                        &mut variables[literal.v_id].negative_watch_occurrences,
                        working_clause_vec,
                    ),
                    false => std::mem::replace(
                        &mut variables[literal.v_id].positive_watch_occurrences,
                        working_clause_vec,
                    ),
                };

                watch_status
            }
        }
        Err(ValuationStatus::Match) => match source {
            LiteralSource::StoredClause(_) => {
                // A literal may be implied by multiple clauses, so there's no need to panic
                // rather, there's no need to do anything at all
                WatchStatus::TwoNone
            }
            _ => {
                log::error!("Attempt to restate {} via {:?}", literal, source);
                panic!("Attempt to restate {} via {:?}", literal, source)
            }
        },
        Err(ValuationStatus::Conflict) => {
            log::error!("Conflict when updating {literal} via {:?}", source);
            panic!("Conflict when updating {literal} via {:?}", source);
        }
        Err(_) => todo!(),
    }
}
