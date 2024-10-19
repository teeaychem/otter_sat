use crate::{
    context::{store::ClauseKey, Context, Result, Status},
    procedures::hobson_choices,
    structures::{
        clause::stored::Watch,
        level::LevelIndex,
        literal::{Literal, Source},
        variable::{variable_store::VariableStore, VariableId},
    },
};

use rand::Rng;

impl Context {
    #[allow(unused_labels)]
    pub fn solve(&mut self) -> Result {
        let this_total_time = std::time::Instant::now();

        let mut last_valuation = vec![None; self.variables.len()];

        if self.config.hobson_choices {
            self.set_hobson();
        }

        // store parts of config for use inside the loop
        let decay_factor = self.config.decay_factor;
        let decay_frequency = self.config.decay_frequency;
        let vsids_variant = self.config.vsids_variant;
        let time_limit = self.config.time_limit;
        let stopping_criteria = self.config.stopping_criteria;
        let luby_constant = self.config.luby_constant;
        let reduction_allowed = self.config.reduction_allowed;
        let restart_allowed = self.config.restarts_allowed;
        let polarity_lean = self.config.polarity_lean;
        let glue_strength = self.config.glue_strength;
        let activity = self.config.activity_conflict;
        let subsumption = self.config.subsumption;
        let random_choice_frequency = self.config.random_choice_frequency;

        'main_loop: loop {
            self.iterations += 1;

            self.time = this_total_time.elapsed();
            if time_limit.is_some_and(|limit| self.time > limit) {
                return Result::Unknown;
            }

            'literal_consequences: while let Some(literal) = self.consequence_q.pop_front() {
                self.update_watches(literal);
                if let Some(conflict_key) = self.examine_consequences_of(literal) {
                    match self.conflict_analysis(
                        conflict_key,
                        vsids_variant,
                        stopping_criteria,
                        activity,
                        subsumption,
                    ) {
                        Status::NoSolution => return Result::Unsatisfiable(conflict_key),
                        Status::MissedImplication => continue 'main_loop,
                        Status::AssertingClause => {
                            for variable in &self.variables {
                                last_valuation[variable.index()] = variable.polarity();
                            }

                            self.conflicts += 1;
                            self.conflicts_since_last_forget += 1;
                            self.conflicts_since_last_reset += 1;

                            if self.conflicts % decay_frequency == 0 {
                                for variable in &self.variables {
                                    variable.multiply_activity(decay_factor);
                                }
                            }

                            self.reductions_and_restarts(
                                reduction_allowed,
                                restart_allowed,
                                luby_constant,
                                glue_strength,
                            );
                            continue 'main_loop;
                        }
                    }
                }
            }

            match self.get_unassigned(random_choice_frequency) {
                Some(choice_index) => {
                    self.process_choice(choice_index, &last_valuation, polarity_lean);
                    continue 'main_loop;
                }
                None => return Result::Satisfiable,
            }
        }
    }

    /*
    let level_index = match source {
            Source::Choice | Source::Clause(_) => &self.levels.len() - 1,
            Source::Assumption | Source::HobsonChoice | Source::Resolution(_) => 0,
            };
     */
    pub fn literal_update(&mut self, literal: Literal, level_index: LevelIndex, source: Source) {
        log::trace!("{literal} from {source:?}");
        self.variables.set_value(literal, level_index);
        unsafe {
            self.levels
                .get_unchecked_mut(level_index)
                .record_literal(literal, source);
        };
    }

    pub fn update_watches(&mut self, literal: Literal) {
        let not_watch_witness = |literal: Literal| {
            let the_variable = self.variables.get_unsafe(literal.index());
            match the_variable.polarity() {
                None => true,
                Some(found_polarity) => found_polarity != literal.polarity(),
            }
        };

        let variable = self.variables.get_unsafe(literal.index());

        // process whether any change to the watch literals is required
        let list_polarity = !literal.polarity();

        let mut index = 0;
        let mut length = variable.occurrence_length(list_polarity);

        while index < length {
            let working_key = variable.occurrence_key_at_index(list_polarity, index);
            let working_clause = self.stored_clauses.retreive_mut(working_key);
            match working_clause {
                None => {
                    variable.remove_occurrence_at_index(list_polarity, index);
                    length -= 1;
                }
                Some(stored_clause) => {
                    let watched_a = stored_clause.get_watch(Watch::A);
                    let watched_b = stored_clause.get_watch(Watch::B);

                    if variable.id() == watched_a.v_id() {
                        if not_watch_witness(watched_b) {
                            stored_clause.update_watch(Watch::A, &self.variables);
                        }
                        index += 1;
                    } else if variable.id() == watched_b.v_id() {
                        if not_watch_witness(watched_a) {
                            stored_clause.update_watch(Watch::B, &self.variables);
                        }
                        index += 1;
                    } else {
                        variable.remove_occurrence_at_index(list_polarity, index);
                        length -= 1;
                    }
                }
            }
        }
    }

    pub fn literal_set_from_vec(&mut self, choices: Vec<VariableId>) {
        for v_id in choices {
            let the_literal = Literal::new(v_id, false);
            self.literal_update(the_literal, 0, Source::HobsonChoice);
            self.consequence_q.push_back(the_literal);
        }
    }

    /*
    todo: transfer to valuation store
    return the new_observations and import these to the context there
    though, use a buffer of variable size to avoid mallocs
    this then is a step toward a better handling of unsafe as the borrow from valuation store while updating other valuations in the store is clearer
     */
    fn examine_consequences_of(&mut self, literal: Literal) -> Option<ClauseKey> {
        let the_variable = self.variables.get_unsafe(literal.index());
        let borrowed_occurrences = {
            match literal.polarity() {
                true => unsafe { &mut *the_variable.negative_occurrences.get() },
                false => unsafe { &mut *the_variable.positive_occurrences.get() },
            }
        };

        let level_index = self.level().index();

        let mut index = 0;
        let mut length = borrowed_occurrences.len();
        let mut new_observations = Vec::with_capacity(length);

        while index < length {
            let clause_key = unsafe { *borrowed_occurrences.get_unchecked(index) };

            let stored_clause = self.stored_clauses.retreive(clause_key);

            let watch_a = stored_clause.get_watch(Watch::A);
            let watch_b = stored_clause.get_watch(Watch::B);

            if watch_a.v_id() != literal.v_id() && watch_b.v_id() != literal.v_id() {
                borrowed_occurrences.swap_remove(index);
                length -= 1;
            } else {
                // the compiler prefers the conditional matches
                index += 1;
                let a_value = self.variables.polarity_of(watch_a.index());
                let b_value = self.variables.polarity_of(watch_b.index());

                match (a_value, b_value) {
                    (None, None) => {}
                    (Some(a), None) if a == watch_a.polarity() => {}
                    (Some(_), None) => {
                        self.variables.set_value(watch_b, level_index);
                        new_observations
                            .push((Source::Clause(stored_clause.node_index()), watch_b));
                        self.consequence_q.push_back(watch_b);
                    }
                    (None, Some(b)) if b == watch_b.polarity() => {}
                    (None, Some(_)) => {
                        self.variables.set_value(watch_a, level_index);
                        new_observations
                            .push((Source::Clause(stored_clause.node_index()), watch_a));
                        self.consequence_q.push_back(watch_a);
                    }
                    (Some(a), Some(b)) if a == watch_a.polarity() || b == watch_b.polarity() => {}
                    (Some(_), Some(_)) => {
                        // clean the watch lists while clearing the q
                        unsafe {
                            self.levels
                                .get_unchecked_mut(level_index)
                                .extend_observations(new_observations);
                        };
                        self.clear_queued_consequences();
                        return Some(clause_key);
                    }
                }
            }
        }
        unsafe {
            self.levels
                .get_unchecked_mut(level_index)
                .extend_observations(new_observations);
        };
        None
    }

    // lazy removals as implemented allow the lists to get quite messy if not kept clean
    fn clear_queued_consequences(&mut self) {
        while let Some(literal) = self.consequence_q.pop_front() {
            let occurrences = {
                let the_variable = self.variables.get_unsafe(literal.index());
                match literal.polarity() {
                    true => unsafe { &mut *the_variable.negative_occurrences.get() },
                    false => unsafe { &mut *the_variable.positive_occurrences.get() },
                }
            };

            let mut index = 0;
            let mut length = occurrences.len();

            while index < length {
                let clause_key = unsafe { *occurrences.get_unchecked(index) };

                match self.stored_clauses.retreive_carefully(clause_key) {
                    Some(stored_clause) => {
                        let watch_a = stored_clause.get_watch(Watch::A);
                        let watch_b = stored_clause.get_watch(Watch::B);

                        if watch_a.v_id() != literal.v_id() && watch_b.v_id() != literal.v_id() {
                            occurrences.swap_remove(index);
                            length -= 1;
                        } else {
                            index += 1;
                        }
                    }
                    None => {
                        occurrences.swap_remove(index);
                        length -= 1;
                    }
                }
            }
        }
    }

    fn reductions_and_restarts(
        &mut self,
        reduction_allowed: bool,
        restart_allowed: bool,
        u: usize,
        glue_strength: usize,
    ) {
        if self.it_is_time_to_reduce(u) {
            if let Some(window) = &self.window {
                self.update_stats(window);
                window.flush();
            }

            if reduction_allowed {
                log::debug!(target: "forget", "Forget @r {}", self.restarts);
                self.stored_clauses.reduce(glue_strength);
            }

            if restart_allowed {
                self.backjump(0);
                self.restarts += 1;
                self.conflicts_since_last_forget = 0;
            }
        }
    }

    fn process_choice(&mut self, index: usize, last_val: &[Option<bool>], polarity_lean: f64) {
        log::trace!(
            "Choice: {index} @ {} with activity {}",
            self.level().index(),
            self.variables.get_unsafe(index).activity()
        );
        let level_index = self.add_fresh_level();
        let choice_literal = {
            let id = index as VariableId;
            match last_val[index] {
                Some(polarity) => Literal::new(id, polarity),
                None => Literal::new(id, rand::thread_rng().gen_bool(polarity_lean)),
            }
        };
        self.literal_update(choice_literal, level_index, Source::Choice);
        self.consequence_q.push_back(choice_literal);
    }

    fn set_hobson(&mut self) {
        let (f, t) = hobson_choices(self.stored_clauses.clauses());
        self.literal_set_from_vec(f);
        self.literal_set_from_vec(t);
    }
}
