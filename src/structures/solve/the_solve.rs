use crate::procedures::hobson_choices;
use crate::structures::{
    clause::{
        stored_clause::{ClauseStatus, StoredClause, Watch, WatchStatus, WatchUpdateEnum},
        Clause,
    },
    level::Level,
    literal::{Literal, LiteralSource},
    solve::{
        clause_store::{retreive, ClauseKey},
        config::{
            config_glue_strength, config_hobson, config_restarts_allowed, config_show_assignment,
            config_show_core, config_show_stats, config_time_limit,
        },
        stats::SolveStats,
        ClauseStore, Solve, {SolveResult, SolveStatus},
    },
    valuation::{Valuation, ValuationStatus},
    variable::Variable,
};

impl Solve {
    #[allow(unused_labels)]
    pub fn do_solve(&mut self) -> (SolveResult, SolveStats) {
        let this_total_time = std::time::Instant::now();

        let mut stats = SolveStats::new();

        if config_hobson() {
            let (f, t) = hobson_choices(self.clauses());
            self.literal_set_from_vec(f);
            self.literal_set_from_vec(t);
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

            let mut found_conflict = None;

            let this_implication_time = std::time::Instant::now();
            'propagation_loop: while let Some(literal) = self.watch_q.pop_front() {
                let the_variable = &self.variables[literal.v_id];

                let borrowed_occurrences = match literal.polarity {
                    true => the_variable.take_occurrence_vec(false),
                    false => the_variable.take_occurrence_vec(true),
                };

                'clause_loop: for clause_key in borrowed_occurrences.iter().cloned() {
                    let stored_clause = retreive(&self.clauses_stored, clause_key);

                    match stored_clause.watch_choices(&self.valuation) {
                        ClauseStatus::Entails(consequent) => {
                            literal_update(
                                consequent,
                                LiteralSource::StoredClause(clause_key),
                                &mut self.levels,
                                &self.variables,
                                &mut self.valuation,
                                &self.clauses_stored,
                            );
                            self.watch_q.push_back(consequent);
                        }
                        ClauseStatus::Conflict => {
                            found_conflict = Some(clause_key);
                            self.watch_q.clear();
                            break 'clause_loop;
                        }
                        ClauseStatus::Unsatisfied => (),
                        ClauseStatus::Satisfied => (),
                    }
                }
                match literal.polarity {
                    true => the_variable.restore_occurrence_vec(false, borrowed_occurrences),
                    false => the_variable.restore_occurrence_vec(true, borrowed_occurrences),
                };
            }
            stats.implication_time += this_implication_time.elapsed();

            match found_conflict {
                None => {
                    let this_choice_time = std::time::Instant::now();
                    if let Some(available_v_id) = self.most_active_none(&self.valuation) {
                        if self.it_is_time_to_reduce() {
                            log::debug!(target: "forget", "{stats} @ {}", self.forgets);
                            let this_reduction_time = std::time::Instant::now();
                            if config_restarts_allowed() {
                                {
                                    // TODO: figure some improvement…
                                    let mut keys_to_drop = vec![];
                                    for (k, v) in &self.clauses_stored.learnt_clauses {
                                        if v.lbd() > config_glue_strength() {
                                            keys_to_drop.push(k);
                                        }
                                    }
                                    for key in keys_to_drop {
                                        self.drop_learnt_clause_by_swap(ClauseKey::Learnt(key))
                                    }
                                }
                                self.watch_q.clear();
                                self.backjump(0);
                            }
                            self.forgets += 1;
                            self.conflicts_since_last_forget = 0;
                            log::debug!(target: "forget", "Reduced to: {}", self.clauses_stored.learnt_clauses.len());

                            stats.reduction_time += this_reduction_time.elapsed();
                        }

                        log::trace!(
                            "Choice: {available_v_id} @ {} with activity {}",
                            self.current_level().index(),
                            self.variables[available_v_id].activity()
                        );
                        let _new_level = self.add_fresh_level();
                        let choice_literal = Literal::new(available_v_id, false);

                        literal_update(
                            choice_literal,
                            LiteralSource::Choice,
                            &mut self.levels,
                            &self.variables,
                            &mut self.valuation,
                            &self.clauses_stored,
                        );
                        self.watch_q.push_back(choice_literal);

                        stats.choice_time += this_choice_time.elapsed();
                        continue 'main_loop;
                    } else {
                        result = SolveResult::Satisfiable;
                        stats.choice_time += this_choice_time.elapsed();
                        break 'main_loop;
                    }
                }
                Some(clause_key) => {
                    self.watch_q.clear();
                    let this_unsat_time = std::time::Instant::now();

                    let conflict_clause = retreive(&self.clauses_stored, clause_key);

                    // notice_conflict
                    {
                        self.conflicts += 1;
                        self.conflicts_since_last_forget += 1;
                        if self.conflicts % 2_usize.pow(9) == 0 {
                            for variable in &self.variables {
                                variable.divide_activity(1.2)
                            }
                        }

                        for variable in conflict_clause.variables() {
                            self.variables[variable].add_activity(2.0);
                        }
                    }

                    let analysis_result = self.attempt_fix(clause_key);
                    stats.conflicts += 1;
                    stats.unsat_time += this_unsat_time.elapsed();
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
                config_show_assignment().then(|| {
                    println!(
                        "c ASSIGNMENT: {}",
                        self.valuation.to_vec().as_display_string(self)
                    )
                });
            }
            SolveResult::Unsatisfiable => {
                config_show_core().then(|| self.core());
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
    stored_clauses: &ClauseStore,
) {
    let variable = &variables[literal.v_id];
    variable.add_activity(1.0);

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

            let mut working_clause_vec = match literal.polarity {
                true => variable.take_occurrence_vec(false),
                false => variable.take_occurrence_vec(true),
            };

            let mut index = 0;
            let mut length = working_clause_vec.len();
            while index < length {
                let clause_key = working_clause_vec[index];

                let stored_clause = retreive(stored_clauses, clause_key);

                let the_watch = match stored_clause.literal_of(Watch::A).v_id == literal.v_id {
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

            match literal.polarity {
                true => variable.restore_occurrence_vec(false, working_clause_vec),
                false => variable.restore_occurrence_vec(true, working_clause_vec),
            };
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
        1 => match val.of_v_id(
            stored_clause
                .literal_at(stored_clause.get_watch(Watch::A))
                .v_id,
        ) {
            None => WatchStatus::SameImplication,
            Some(_) => WatchStatus::SameSatisfied,
        },
        _ => {
            macro_rules! update_the_watch_to {
                ($idx:expr) => {
                    match chosen_watch {
                        Watch::A => {
                            stored_clause.set_watch(Watch::A, $idx);
                            let watched_a = stored_clause.literal_of(Watch::A);
                            variables[watched_a.v_id]
                                .watch_added(stored_clause.key, watched_a.polarity)
                        }
                        Watch::B => {
                            stored_clause.set_watch(Watch::B, $idx);
                            let watched_b = stored_clause.literal_of(Watch::B);
                            variables[watched_b.v_id]
                                .watch_added(stored_clause.key, watched_b.polarity)
                        }
                    }
                };
            }

            let watched_x_value = val.of_v_id(
                stored_clause
                    .literal_at(stored_clause.get_watch(chosen_watch))
                    .v_id,
            );

            let watched_y_literal = match chosen_watch {
                Watch::A => stored_clause.literal_at(stored_clause.get_watch(Watch::B)),
                Watch::B => stored_clause.literal_at(stored_clause.get_watch(Watch::A)),
            };

            let watched_y_value = val.of_v_id(watched_y_literal.v_id);

            if let Some(_current_x_value) = watched_x_value {
                match stored_clause.some_none_or_else_witness_idx(val, watched_y_literal.v_id) {
                    WatchUpdateEnum::Witness(idx) => {
                        if watched_y_value.is_some_and(|p| p == watched_y_literal.polarity) {
                            WatchStatus::SameSatisfied
                        } else {
                            update_the_watch_to!(idx);
                            WatchStatus::NewSatisfied
                        }
                    }
                    WatchUpdateEnum::None(idx) => {
                        update_the_watch_to!(idx);

                        match watched_y_value {
                            None => WatchStatus::NewTwoNone,
                            Some(p) if p == watched_y_literal.polarity => WatchStatus::NewSatisfied,
                            _ => WatchStatus::NewImplication,
                        }
                    }
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
