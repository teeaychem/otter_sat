use crate::procedures::hobson_choices;
use crate::structures::{
    clause::{
        stored_clause::{ClauseStatus, StoredClause, Watch, WatchUpdate},
        Clause,
    },
    level::Level,
    literal::{Literal, LiteralSource},
    solve::{
        config, retreive, retreive_mut,
        stats::SolveStats,
        ClauseKey, ClauseStore, Solve, {SolveResult, SolveStatus},
    },
    valuation::{Valuation, ValuationWindow},
    variable::{Variable, VariableId},
};

#[allow(unused_imports)] // used in timing macros
use crate::structures::solve::stats;

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

            'propagation_loop: while let Some(literal) = self.watch_q.pop_front() {
                let the_variable = &self.variables[literal.v_id()];

                unsafe {
                    let borrowed_occurrences = match literal.polarity {
                        true => &mut *the_variable.negative_watch_occurrences.get(),
                        false => &mut *the_variable.positive_watch_occurrences.get(),
                    };

                    let mut index = 0;
                    let mut length = borrowed_occurrences.len();

                    'clause_loop: while index < length {
                        let clause_key = *borrowed_occurrences.get_unchecked(index);

                        let stored_clause =
                            retreive(&self.formula_clauses, &self.learnt_clauses, clause_key);

                        let watch_choices =
                            stored_clause.watch_status(&self.valuation, the_variable.id());

                        let clause_key = stored_clause.key();

                        match watch_choices {
                            ClauseStatus::Missing => {
                                borrowed_occurrences.swap_remove(index);
                                length -= 1;
                            }
                            ClauseStatus::Implies(consequent) => {
                                literal_update(
                                    consequent,
                                    LiteralSource::StoredClause(clause_key),
                                    &mut self.levels,
                                    &self.variables,
                                    &mut self.valuation,
                                    &mut self.formula_clauses,
                                    &mut self.learnt_clauses,
                                );
                                self.watch_q.push_back(consequent);
                                index += 1;
                            }
                            ClauseStatus::Conflict => {
                                found_conflict = Some(clause_key);
                                self.watch_q.clear();
                                break 'clause_loop;
                            }
                            ClauseStatus::Unsatisfied | ClauseStatus::Satisfied => {
                                index += 1;
                            }
                        }
                    }
                }
            }

            match found_conflict {
                None => {
                    if unsafe { config::REDUCTION_ALLOWED } && self.it_is_time_to_reduce() {
                        log::debug!(target: "forget", "{stats} @r {}", self.restarts);

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
                            self.drop_learnt_clause(ClauseKey::Learnt(key))
                        }

                        log::debug!(target: "forget", "Reduced to: {}", self.learnt_clauses.len());
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
                                Literal::new(available_v_id as VariableId, polarity)
                            } else {
                                Literal::new(available_v_id as VariableId, false)
                            }
                        } else {
                            Literal::new(available_v_id as VariableId, false)
                        };
                        literal_update(
                            choice_literal,
                            LiteralSource::Choice,
                            &mut self.levels,
                            &self.variables,
                            &mut self.valuation,
                            &mut self.formula_clauses,
                            &mut self.learnt_clauses,
                        );
                        self.watch_q.push_back(choice_literal);
                        continue 'main_loop;
                    } else {
                        result = SolveResult::Satisfiable;
                        break 'main_loop;
                    }
                }
                Some(clause_key) => {
                    self.conflicts += 1;
                    self.conflicts_since_last_forget += 1;
                    self.conflicts_since_last_reset += 1;

                    if self.conflicts % config::DECAY_FREQUENCY == 0 {
                        for variable in &self.variables {
                            variable.multiply_activity(config::DECAY_FACTOR);
                        }
                    }

                    let analysis_result = self.attempt_fix(clause_key);
                    stats.conflicts += 1;
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
                    println!("c ASSIGNMENT: {}", self.valuation.as_display_string(self))
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

#[allow(clippy::too_many_arguments)]
pub fn literal_update(
    literal: Literal,
    source: LiteralSource,
    levels: &mut [Level],
    variables: &[Variable],
    valuation: &mut ValuationWindow,
    formula_clauses: &mut ClauseStore,
    learnt_clauses: &mut ClauseStore,
) {
    let literal_v_id = literal.v_id;

    let variable = unsafe { variables.get_unchecked(literal_v_id as usize) };

    // update the valuation and match the result
    valuation.set_value(literal);

    log::trace!("Set {source:?}: {literal}");
    // if update occurrs, make records at the relevant level

    unsafe {
        {
            let level_index = match &source {
                LiteralSource::Choice | LiteralSource::StoredClause(_) => levels.len() - 1,
                LiteralSource::Assumption
                | LiteralSource::HobsonChoice
                | LiteralSource::Resolution(_) => 0,
            };
            variable.set_decision_level(level_index);
            levels
                .get_unchecked_mut(level_index)
                .record_literal(literal, &source);
        }

        // and, process whether any change to the watch literals is required
        let working_clause_vec = match literal.polarity {
            true => &mut *variable.negative_watch_occurrences.get(),
            false => &mut *variable.positive_watch_occurrences.get(),
        };

        let mut index = 0;
        let mut length = working_clause_vec.len();

        while index < length {
            if let Some(stored_clause) = retreive_mut(
                formula_clauses,
                learnt_clauses,
                *working_clause_vec.get_unchecked(index),
            ) {
                let (a_v_id, a_polarity) = stored_clause.get_watched_split(Watch::A);
                let (b_v_id, b_polarity) = stored_clause.get_watched_split(Watch::B);

                if a_v_id == literal_v_id {
                    if !watch_witnesses(valuation, b_v_id, b_polarity) {
                        process_watches(valuation, variables, stored_clause, Watch::A);
                    }
                    index += 1;
                } else if b_v_id == literal_v_id {
                    if !watch_witnesses(valuation, a_v_id, a_polarity) {
                        process_watches(valuation, variables, stored_clause, Watch::B);
                    }
                    index += 1;
                } else {
                    working_clause_vec.swap_remove(index);
                    length -= 1;
                }
            } else {
                working_clause_vec.swap_remove(index);
                length -= 1;
            }
        }
    }
}

fn watch_witnesses(valuation: &ValuationWindow, v_id: VariableId, polarity: bool) -> bool {
    if let Some(p) = valuation.of_v_id(v_id) {
        p == polarity
    } else {
        false
    }
}

fn process_watches(
    valuation: &ValuationWindow,
    variables: &[Variable],
    stored_clause: &mut StoredClause,
    chosen_watch: Watch,
) {
    match stored_clause.update_watch(chosen_watch, valuation) {
        WatchUpdate::Update(v_id, polarity) => {
            unsafe {
                variables
                    .get_unchecked(v_id as usize)
                    .watch_added(stored_clause.key(), polarity);
            };
        }
        WatchUpdate::NoUpdate => {}
    }
}

impl Solve {
    pub fn literal_set_from_vec(&mut self, choices: Vec<VariableId>) {
        choices.iter().for_each(|&v_id| {
            let the_literal = Literal::new(v_id, false);
            literal_update(
                the_literal,
                LiteralSource::HobsonChoice,
                &mut self.levels,
                &self.variables,
                &mut self.valuation,
                &mut self.formula_clauses,
                &mut self.learnt_clauses,
            );
            self.watch_q.push_back(the_literal);
        });
    }
}
