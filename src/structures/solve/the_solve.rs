use crate::procedures::hobson_choices;
use crate::structures::clause::stored_clause;
use crate::structures::{
    clause::{
        stored_clause::{
            ClauseKey, ClauseStatus, StoredClause, Watch, WatchStatus, WatchUpdateEnum,
        },
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

use slotmap::{DefaultKey, SlotMap};

impl Solve {
    #[allow(unused_labels)]
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

            let mut found_conflict = None;

            let this_implication_time = std::time::Instant::now();
            'propagation_loop: while let Some(literal) = self.watch_q.pop_front() {
                let temprary_clause_vec = match literal.polarity {
                    true => self.variables[literal.v_id].take_occurrence_vec(false),
                    false => self.variables[literal.v_id].take_occurrence_vec(true),
                };

                'clause_loop: for clause_key in &temprary_clause_vec {
                    let stored_clause = match clause_key {
                        stored_clause::ClauseKey::Formula(key) => &self.formula_clauses[*key],
                        stored_clause::ClauseKey::Learnt(key) => &self.learnt_clauses[*key],
                    };

                    match stored_clause.watch_choices(&self.valuation) {
                        ClauseStatus::Entails(consequent) => {
                            literal_update(
                                consequent,
                                LiteralSource::StoredClause(*clause_key),
                                &mut self.levels,
                                &self.variables,
                                &mut self.valuation,
                                &self.formula_clauses,
                                &self.learnt_clauses,
                            );
                            self.watch_q.push_back(consequent);
                        }
                        ClauseStatus::Conflict => {
                            found_conflict = Some(*clause_key);
                            self.watch_q.clear();
                            break 'clause_loop;
                        }
                        ClauseStatus::Unsatisfied => (),
                        ClauseStatus::Satisfied => (),
                    }
                }
                match literal.polarity {
                    true => self.variables[literal.v_id]
                        .restore_occurrence_vec(false, temprary_clause_vec),
                    false => self.variables[literal.v_id]
                        .restore_occurrence_vec(true, temprary_clause_vec),
                };
            }
            stats.implication_time += this_implication_time.elapsed();

            match found_conflict {
                None => {
                    if let Some(available_v_id) = self.most_active_none(&self.valuation) {
                        if self.it_is_time_to_reduce() {
                            log::debug!(target: "forget", "{stats} @ {}", self.forgets);
                            let this_reduction_time = std::time::Instant::now();
                            if config_restarts_allowed() {
                                {
                                    let mut keys_to_drop = vec![];
                                    for (k, v) in &self.learnt_clauses {
                                        if v.lbd() > config_glue_strength() {
                                            keys_to_drop.push(k);
                                        }
                                    }
                                    for k in keys_to_drop {
                                        self.drop_learnt_clause_by_swap(ClauseKey::Learnt(k))
                                    }
                                }
                                self.watch_q.clear();
                                self.backjump(0);
                            }
                            self.forgets += 1;
                            self.conflicts_since_last_forget = 0;
                            log::debug!(target: "forget", "Reduced to: {}", self.learnt_clauses.len());

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
                            &self.variables,
                            &mut self.valuation,
                            &self.formula_clauses,
                            &self.learnt_clauses,
                        );
                        self.watch_q.push_back(the_literal);

                        stats.choice_time += this_choice_time.elapsed();

                        continue 'main_loop;
                    } else {
                        result = SolveResult::Satisfiable;
                        break 'main_loop;
                    }
                }
                Some(clause_key) => {
                    self.watch_q.clear();
                    let this_unsat_time = std::time::Instant::now();

                    let stored_clause = match clause_key {
                        stored_clause::ClauseKey::Formula(key) => &self.formula_clauses[key],
                        stored_clause::ClauseKey::Learnt(key) => &self.learnt_clauses[key],
                    };

                    // notice_conflict
                    {
                        self.conflicts += 1;
                        self.conflicts_since_last_forget += 1;
                        if self.conflicts % 2_usize.pow(10) == 0 {
                            for variable in &self.variables {
                                variable.divide_activity(2.0)
                            }
                        }

                        for literal in stored_clause.variables() {
                            self.variables[literal].add_activity(2.0);
                        }
                    }

                    let analysis_result = self.attempt_fix(clause_key);
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

pub fn literal_update(
    literal: Literal,
    source: LiteralSource,
    levels: &mut [Level],
    vars: &[Variable],
    valuation: &mut impl Valuation,
    formula_clauses: &SlotMap<DefaultKey, StoredClause>,
    learnt_clauses: &SlotMap<DefaultKey, StoredClause>,
) {
    let variable = &vars[literal.v_id];
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

            // TODO: this is slower than duplicating the following loop for both +/- occurrence vecs
            // Though, as a function is not viable given the use of variables in the process functions,
            // this while unstable this allows updating code in only one place
            let mut working_clause_vec = match literal.polarity {
                true => vars[literal.v_id].take_occurrence_vec(false),
                false => vars[literal.v_id].take_occurrence_vec(true),
            };

            let mut index = 0;
            let mut length = working_clause_vec.len();
            while index < length {
                let clause_key = &working_clause_vec[index];

                let stored_clause = match clause_key {
                    ClauseKey::Formula(key) => &formula_clauses[*key],
                    ClauseKey::Learnt(key) => &learnt_clauses[*key],
                };

                let the_watch = match stored_clause.watched_a().v_id == literal.v_id {
                    true => Watch::A,
                    false => Watch::B,
                };

                match process_watches(valuation, vars, stored_clause, the_watch) {
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
                true => vars[literal.v_id].restore_occurrence_vec(false, working_clause_vec),
                false => vars[literal.v_id].restore_occurrence_vec(true, working_clause_vec),
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
                            variables[watched_a.v_id]
                                .watch_added(stored_clause.key, watched_a.polarity)
                        }
                        Watch::B => {
                            stored_clause.update_watch_b($a);
                            let watched_b = stored_clause.watched_b();
                            variables[watched_b.v_id]
                                .watch_added(stored_clause.key, watched_b.polarity)
                        }
                    }
                };
            }

            let watched_x_literal = stored_clause.literal_at(stored_clause.get_watch(chosen_watch));

            let watched_y_literal = match chosen_watch {
                Watch::A => stored_clause.literal_at(stored_clause.get_watch(Watch::B)),
                Watch::B => stored_clause.literal_at(stored_clause.get_watch(Watch::A)),
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
