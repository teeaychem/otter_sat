use crate::procedures::hobson_choices;
use crate::structures::{
    clause::{stored_clause::Watch, Clause},
    level::Level,
    literal::{Literal, LiteralSource},
    solve::{
        config, retreive, retreive_mut,
        stats::SolveStats,
        ClauseStore, Solve, {SolveResult, SolveStatus},
    },
    valuation::Valuation,
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
                .formula_clauses
                .iter()
                .chain(&self.learnt_clauses)
                .map(|(_, sc)| sc)
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
                let the_variable = unsafe { &self.variables.get_unchecked(literal.index()) };

                let borrowed_occurrences = unsafe {
                    match literal.polarity() {
                        true => &mut *the_variable.negative_watch_occurrences.get(),
                        false => &mut *the_variable.positive_watch_occurrences.get(),
                    }
                };

                let mut index = 0;
                let mut length = borrowed_occurrences.len();

                'clause_loop: while index < length {
                    let clause_key = unsafe { *borrowed_occurrences.get_unchecked(index) };

                    let stored_clause =
                        retreive(&self.formula_clauses, &self.learnt_clauses, clause_key);

                    let the_id = literal.v_id();

                    let watch_a = stored_clause.get_watched(Watch::A);
                    let watch_b = stored_clause.get_watched(Watch::B);

                    if watch_a.v_id() != the_id && watch_b.v_id() != the_id {
                        borrowed_occurrences.swap_remove(index);
                        length -= 1;
                    } else {
                        // the compiler prefers the conditional matches
                        index += 1;
                        let a_value = self.valuation.of_index(watch_a.index());
                        let b_value = self.valuation.of_index(watch_b.index());
                        match (a_value, b_value) {
                            (None, None) => {}
                            (Some(a), None) if a == watch_a.polarity() => {}
                            (Some(_), None) => {
                                literal_update(
                                    watch_b,
                                    LiteralSource::StoredClause(clause_key),
                                    &mut self.levels,
                                    &self.variables,
                                    &mut self.valuation,
                                    &mut self.formula_clauses,
                                    &mut self.learnt_clauses,
                                );
                                self.watch_q.push_back(watch_b);
                            }
                            (None, Some(b)) if b == watch_b.polarity() => {}
                            (None, Some(_)) => {
                                literal_update(
                                    watch_a,
                                    LiteralSource::StoredClause(clause_key),
                                    &mut self.levels,
                                    &self.variables,
                                    &mut self.valuation,
                                    &mut self.formula_clauses,
                                    &mut self.learnt_clauses,
                                );
                                self.watch_q.push_back(watch_a);
                            }
                            (Some(a), Some(b))
                                if a == watch_a.polarity() || b == watch_b.polarity() => {}
                            (Some(_), Some(_)) => {
                                found_conflict = Some(clause_key);
                                self.watch_q.clear();
                                break 'clause_loop;
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
                            self.learnt_clauses.remove(key);
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
                            self.level().index(),
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
    valuation: &mut impl Valuation,
    formula_clauses: &mut ClauseStore,
    learnt_clauses: &mut ClauseStore,
) {
    let literal_v_id = literal.v_id();

    let variable = unsafe { variables.get_unchecked(literal_v_id as usize) };

    // update the valuation and match the result
    valuation.set_value(literal);

    log::trace!("{literal} from {source:?}");
    // if update occurrs, make records at the relevant level

    {
        let level_index = match &source {
            LiteralSource::Choice | LiteralSource::StoredClause(_) => levels.len() - 1,
            LiteralSource::Assumption
            | LiteralSource::HobsonChoice
            | LiteralSource::Resolution(_) => 0,
        };
        variable.set_decision_level(level_index);
        unsafe {
            levels
                .get_unchecked_mut(level_index)
                .record_literal(literal, &source)
        };
    }

    // and, process whether any change to the watch literals is required
    let working_clause_vec = unsafe {
        match literal.polarity() {
            true => &mut *variable.negative_watch_occurrences.get(),
            false => &mut *variable.positive_watch_occurrences.get(),
        }
    };

    let mut index = 0;
    let mut length = working_clause_vec.len();

    while index < length {
        let working_clause = unsafe {
            retreive_mut(
                formula_clauses,
                learnt_clauses,
                *working_clause_vec.get_unchecked(index),
            )
        };
        match working_clause {
            None => {
                working_clause_vec.swap_remove(index);
                length -= 1;
            }
            Some(stored_clause) => {
                let watched_a = stored_clause.get_watched(Watch::A);
                let watched_b = stored_clause.get_watched(Watch::B);

                if literal_v_id == watched_a.v_id() {
                    if not_watch_witness(valuation, watched_b) {
                        stored_clause.update_watch(Watch::A, valuation, variables);
                    }
                    index += 1;
                } else if literal_v_id == watched_b.v_id() {
                    if not_watch_witness(valuation, watched_a) {
                        stored_clause.update_watch(Watch::B, valuation, variables);
                    }
                    index += 1;
                } else {
                    working_clause_vec.swap_remove(index);
                    length -= 1;
                }
            }
        }
    }
}

fn not_watch_witness(valuation: &impl Valuation, literal: Literal) -> bool {
    match valuation.of_index(literal.index()) {
        Some(p) => p != literal.polarity(),
        None => true,
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
