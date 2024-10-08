use crate::procedures::hobson_choices;
use crate::structures::{
    clause::{
        stored_clause::{ClauseStatus, StoredClause, Watch, WatchStatus, WatchUpdateEnum},
        Clause,
    },
    level::Level,
    literal::{Literal, LiteralSource},
    solve::{
        config, retreive,
        stats::SolveStats,
        ClauseKey, ClauseStore, Solve, {SolveResult, SolveStatus},
    },
    valuation::{Valuation, ValuationStatus},
    variable::Variable,
};

#[allow(unused_imports)] // used in timing macros
use crate::structures::solve::stats;

macro_rules! time_statement {
    ($id:expr, $s:stmt) => {
        #[cfg(feature = "time")]
        let this_time = std::time::Instant::now();
        $s
        #[cfg(feature = "time")]
        unsafe {
            $id += this_time.elapsed();
        }
    }
}

macro_rules! time_block {
    ($id:expr, $b:block) => {
        #[cfg(feature = "time")]
        let this_time = std::time::Instant::now();
        $b
        #[cfg(feature = "time")]
        #[allow(unused_unsafe)]
        unsafe {
            $id += this_time.elapsed();
        }
    }
}

impl Solve {
    #[allow(unused_labels)]
    pub fn do_solve(&mut self) -> (SolveResult, SolveStats) {
        let this_total_time = std::time::Instant::now();

        let mut stats = SolveStats::new();
        let mut last_valuation = None;

        if unsafe { config::HOBSON_CHOICES } {
            let lits = self
                .stored_clauses()
                .map(|stored_clause| stored_clause.literals());
            let (f, t) = hobson_choices(lits);
            self.literal_set_from_vec(f);
            self.literal_set_from_vec(t);
        }

        let result: SolveResult;

        'main_loop: loop {
            stats.total_time = this_total_time.elapsed();
            if let Some(time) = unsafe { config::TIME_LIMIT } {
                if stats.total_time > time {
                    if unsafe { config::SHOW_STATS } {
                        println!("c TIME LIMIT EXCEEDED")
                    };
                    result = SolveResult::Unknown;
                    break 'main_loop;
                }
            }

            stats.iterations += 1;

            let mut found_conflict = None;

            time_block!(stats::PROPAGATION_TIME, {
                'propagation_loop: while let Some((literal, source)) = self.watch_q.pop_front() {
                    let the_variable = &self.variables[literal.v_id];

                    time_block!(stats::LITERAL_UPDATE_TIME, {
                        literal_update(
                            literal,
                            source,
                            &mut self.levels,
                            &self.variables,
                            &mut self.valuation,
                            &self.formula_clauses,
                            &self.learnt_clauses,
                        );
                    });

                    time_statement!(
                        stats::PROP_BORROW_TIME,
                            let borrowed_occurrences = match literal.polarity {
                                true => the_variable.take_occurrence_vec(false),
                                false => the_variable.take_occurrence_vec(true),
                            }
                    );

                    time_block!(stats::CLAUSE_LOOP_TIME, {
                        'clause_loop: for clause_key in borrowed_occurrences.iter().cloned() {
                            time_statement!(stats::GET_STORED_TIME,
                                let stored_clause = retreive(&self.formula_clauses, &self.learnt_clauses, clause_key)
                            );

                            time_statement!(stats::WATCH_CHOICES_TIME,
                                let watch_choices = stored_clause.watch_status(&self.valuation)
                            );

                            match watch_choices {
                                ClauseStatus::Entails(consequent) => {
                                    self.watch_q.push_back((
                                        consequent,
                                        LiteralSource::StoredClause(stored_clause.key),
                                    ));
                                }
                                ClauseStatus::Conflict => {
                                    found_conflict = Some(clause_key);
                                    self.watch_q.clear();
                                    break 'clause_loop;
                                }
                                ClauseStatus::Unsatisfied | ClauseStatus::Satisfied => (),
                            }
                        }
                    });

                    time_block!(stats::PROP_BORROW_TIME, {
                        match literal.polarity {
                            true => {
                                the_variable.restore_occurrence_vec(false, borrowed_occurrences)
                            }
                            false => {
                                the_variable.restore_occurrence_vec(true, borrowed_occurrences)
                            }
                        };
                    });
                }
            });

            match found_conflict {
                None => {
                    #[cfg(feature = "time")]
                    let this_choice_time = std::time::Instant::now();

                    if unsafe { config::REDUCTION_ALLOWED } && self.it_is_time_to_reduce() {
                        log::debug!(target: "forget", "{stats} @r {}", self.restarts);

                        time_block!(stats::REDUCTION_TIME, {
                            // // TODO: figure some improvement…

                            let limit = self.learnt_clauses.len();
                            let mut keys_to_drop = vec![];
                            for (k, v) in &self.learnt_clauses {
                                if keys_to_drop.len() > limit {
                                    break;
                                } else if v.get_set_lbd() > unsafe { config::GLUE_STRENGTH } {
                                    keys_to_drop.push(k);
                                }
                            }

                            for key in keys_to_drop {
                                self.drop_learnt_clause_by_swap(ClauseKey::Learnt(key))
                            }

                            log::debug!(target: "forget", "Reduced to: {}", self.learnt_clauses.len());
                        });
                    }

                    if unsafe { config::RESTARTS_ALLOWED } && self.it_is_time_to_reduce() {
                        last_valuation = Some(self.valuation.clone());
                        self.backjump(0);
                        self.restarts += 1;
                        self.conflicts_since_last_forget = 0;
                    }

                    if let Some(available_v_id) = self.most_active_none(&self.valuation) {
                        log::trace!(
                            "Choice: {available_v_id} @ {} with activity {}",
                            self.current_level().index(),
                            self.variables[available_v_id].activity()
                        );
                        let _new_level = self.add_fresh_level();
                        let choice_literal = if let Some(previous) = &last_valuation {
                            if let Some(polarity) = previous[available_v_id] {
                                Literal::new(available_v_id, polarity)
                            } else {
                                Literal::new(available_v_id, false)
                            }
                        } else {
                            Literal::new(available_v_id, false)
                        };
                        self.watch_q
                            .push_back((choice_literal, LiteralSource::Choice));
                        #[cfg(feature = "time")]
                        unsafe {
                            stats::CHOICE_TIME += this_choice_time.elapsed();
                        }
                        continue 'main_loop;
                    } else {
                        result = SolveResult::Satisfiable;
                        #[cfg(feature = "time")]
                        unsafe {
                            stats::CHOICE_TIME += this_choice_time.elapsed();
                        }
                        break 'main_loop;
                    }
                }
                Some(clause_key) => {
                    #[cfg(feature = "time")]
                    let this_conflict_time = std::time::Instant::now();

                    let conflict_clause =
                        retreive(&self.formula_clauses, &self.learnt_clauses, clause_key);

                    self.conflicts += 1;
                    self.conflicts_since_last_forget += 1;
                    self.conflicts_since_last_reset += 1;
                    if self.conflicts % config::DECAY_FREQUENCY == 0 {
                        for variable in &self.variables {
                            variable.multiply_activity(config::DECAY_FACTOR)
                        }
                    }

                    for variable in conflict_clause.variables() {
                        self.variables[variable].add_activity(config::ACTIVITY_CONFLICT);
                    }

                    let analysis_result = self.attempt_fix(clause_key);
                    stats.conflicts += 1;
                    #[cfg(feature = "time")]
                    unsafe {
                        stats::CONFLICT_TIME += this_conflict_time.elapsed();
                    }
                    match analysis_result {
                        SolveStatus::NoSolution => {
                            result = SolveResult::Unsatisfiable;
                            break 'main_loop;
                        }
                        SolveStatus::AssertingClause => {
                            continue 'main_loop;
                        }
                    }
                }
            }
        }
        // loop exit
        stats.total_time = this_total_time.elapsed();
        match result {
            SolveResult::Satisfiable => {
                if unsafe { config::SHOW_ASSIGNMENT } {
                    println!(
                        "c ASSIGNMENT: {}",
                        self.valuation.to_vec().as_display_string(self)
                    )
                }
            }
            SolveResult::Unsatisfiable => {
                if unsafe { config::SHOW_CORE } {
                    self.core()
                }
            }
            SolveResult::Unknown => {}
        }
        (result, stats)
    }
}

pub fn literal_update(
    literal: Literal,
    source: LiteralSource,
    levels: &mut [Level],
    variables: &[Variable],
    valuation: &mut impl Valuation,
    formula_clauses: &ClauseStore,
    learnt_clauses: &ClauseStore,
) {
    let variable = &variables[literal.v_id];

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
            match literal.polarity {
                true => {
                    let mut working_clause_vec = variable.negative_watch_occurrences.take();
                    let mut index = 0;
                    let mut length = working_clause_vec.len();
                    unsafe {
                        while index < length {
                            let clause_key = *working_clause_vec.get_unchecked(index);

                            let stored_clause =
                                retreive(formula_clauses, learnt_clauses, clause_key);

                            let the_watch =
                                if stored_clause.literal_of(Watch::A).v_id == literal.v_id {
                                    Watch::A
                                } else if stored_clause.literal_of(Watch::B).v_id == literal.v_id {
                                    Watch::B
                                } else {
                                    panic!("oh")
                                };

                            match process_watches(valuation, variables, stored_clause, the_watch) {
                                WatchStatus::SameSatisfied
                                | WatchStatus::SameImplication
                                | WatchStatus::SameConflict => {
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
                    }
                    variable.negative_watch_occurrences.set(working_clause_vec)
                }

                false => {
                    let mut working_clause_vec = variable.positive_watch_occurrences.take();

                    let mut index = 0;
                    let mut length = working_clause_vec.len();
                    unsafe {
                        while index < length {
                            let clause_key = working_clause_vec.get_unchecked(index);

                            let stored_clause =
                                retreive(formula_clauses, learnt_clauses, *clause_key);

                            let the_watch =
                                match stored_clause.literal_of(Watch::A).v_id == literal.v_id {
                                    true => Watch::A,
                                    false => Watch::B,
                                };

                            match process_watches(valuation, variables, stored_clause, the_watch) {
                                WatchStatus::SameSatisfied
                                | WatchStatus::SameImplication
                                | WatchStatus::SameConflict => {
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
                    }
                    variable.positive_watch_occurrences.set(working_clause_vec)
                }
            }
        }
        Err(ValuationStatus::Match) => match source {
            LiteralSource::StoredClause(_) => {
                // A literal may be implied by multiple clauses, so there's no need to do anything
            }
            _ => panic!("Restatement of {} via {:?}", literal, source),
        },
        Err(ValuationStatus::Conflict) => panic!("Conflict given {literal} via {:?}", source),
        Err(_) => todo!(),
    }
}

pub fn process_watches(
    val: &impl Valuation,
    variables: &[Variable],
    stored_clause: &StoredClause,
    chosen_watch: Watch,
) -> WatchStatus {
    match stored_clause.length() {
        1 => match val.of_v_id(stored_clause.get_watched(Watch::A).v_id) {
            None => WatchStatus::SameImplication,
            Some(_) => WatchStatus::SameSatisfied,
        },
        _ => {
            let watched_x_value = val.of_v_id(stored_clause.get_watched(chosen_watch).v_id);

            let watched_y_literal = match chosen_watch {
                Watch::A => stored_clause.get_watched(Watch::B),
                Watch::B => stored_clause.get_watched(Watch::A),
            };

            let watched_y_value = val.of_v_id(watched_y_literal.v_id);

            if let Some(_current_x_value) = watched_x_value {
                time_statement!(stats::NEW_WATCH_TIME,
                let update = stored_clause.some_none_or_else_witness_idx(val, Some(watched_y_literal.v_id),
                   !watched_y_value.is_some_and(|p| p == watched_y_literal.polarity)
                                )
                            );

                match update {
                    WatchUpdateEnum::Witness(idx) | WatchUpdateEnum::None(idx) => unsafe {
                        match chosen_watch {
                            Watch::A => {
                                stored_clause.set_watch(Watch::A, idx);
                                let watched_a = stored_clause.literal_of(Watch::A);
                                variables
                                    .get_unchecked(watched_a.v_id)
                                    .watch_added(stored_clause.key, watched_a.polarity)
                            }
                            Watch::B => {
                                stored_clause.set_watch(Watch::B, idx);
                                let watched_b = stored_clause.literal_of(Watch::B);
                                variables
                                    .get_unchecked(watched_b.v_id)
                                    .watch_added(stored_clause.key, watched_b.polarity)
                            }
                        }
                    },
                    _ => {}
                };

                match update {
                    WatchUpdateEnum::Witness(_) => WatchStatus::NewSatisfied,
                    WatchUpdateEnum::None(_) => match watched_y_value {
                        None => WatchStatus::NewTwoNone,
                        Some(p) if p == watched_y_literal.polarity => WatchStatus::NewSatisfied,
                        _ => WatchStatus::NewImplication,
                    },
                    WatchUpdateEnum::No => match watched_y_value {
                        None => WatchStatus::SameImplication,
                        Some(p) if p == watched_y_literal.polarity => WatchStatus::SameSatisfied,
                        _ => WatchStatus::SameConflict,
                    },
                }
            } else {
                panic!("Process watches without value");
            }
        }
    }
}
