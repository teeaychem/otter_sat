use crate::{
    context::{
        store::ClauseKey,
        Context, {Result, Status},
    },
    procedures::hobson_choices,
    structures::{
        clause::stored::Watch,
        literal::{Literal, Source},
        valuation::Valuation,
        variable::VariableId,
    },
};

use rand::Rng;

impl Context {
    #[allow(unused_labels)]
    pub fn solve(&mut self) -> Result {
        let this_total_time = std::time::Instant::now();

        #[allow(unused_assignments)]
        let mut last_valuation = self.valuation.clone();

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

        'main_loop: loop {
            self.iterations += 1;

            self.time = this_total_time.elapsed();
            if time_limit.is_some_and(|limit| self.time > limit) {
                return Result::Unknown;
            }

            'literal_consequences: while let Some(literal) = self.consequence_q.pop_front() {
                self.update_watches(literal);
                if let Some(conflict_key) = self.examine_consequences_of(literal) {
                    self.conflicts += 1;
                    self.conflicts_since_last_forget += 1;
                    self.conflicts_since_last_reset += 1;
                    last_valuation = self.valuation.clone();

                    if self.conflicts % decay_frequency == 0 {
                        for variable in &self.variables {
                            variable.multiply_activity(decay_factor);
                        }
                    }

                    match self.conflict_analysis(
                        conflict_key,
                        vsids_variant,
                        stopping_criteria,
                        activity,
                    ) {
                        Status::NoSolution => return Result::Unsatisfiable,
                        Status::MissedImplication => continue 'main_loop,
                        Status::AssertingClause => {
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

            match self.most_active_none(&self.valuation) {
                Some(choice_index) => {
                    self.process_choice(choice_index, &last_valuation, polarity_lean);
                    continue 'main_loop;
                }
                None => return Result::Satisfiable,
            }
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn literal_update(&mut self, literal: Literal, source: &Source) {
        // update the valuation and match the result
        self.valuation.set_value(literal);

        let variable = self.get_variable(literal.index());

        log::trace!("{literal} from {source:?}");

        let level_index = match &source {
            Source::Choice | Source::StoredClause(_) => &self.levels.len() - 1,
            Source::Assumption | Source::HobsonChoice | Source::Resolution(_) => 0,
        };
        variable.set_decision_level(level_index);
        unsafe {
            self.levels
                .get_unchecked_mut(level_index)
                .record_literal(literal, source);
        };
    }

    pub fn update_watches(&mut self, literal: Literal) {
        let variable = self.get_variable(literal.index());
        let variable_id = variable.id();
        // and, process whether any change to the watch literals is required
        let working_clause_vec = match literal.polarity() {
            true => unsafe { &mut *variable.negative_occurrences.get() },
            false => unsafe { &mut *variable.positive_occurrences.get() },
        };

        let mut index = 0;
        let mut length = working_clause_vec.len();

        while index < length {
            let working_clause = unsafe {
                self.stored_clauses
                    .retreive_mut(*working_clause_vec.get_unchecked(index))
            };
            match working_clause {
                None => {
                    working_clause_vec.swap_remove(index);
                    length -= 1;
                }
                Some(stored_clause) => {
                    let watched_a = stored_clause.get_watched(Watch::A);
                    let watched_b = stored_clause.get_watched(Watch::B);

                    if variable_id == watched_a.v_id() {
                        if not_watch_witness(&self.valuation, watched_b) {
                            stored_clause.update_watch(Watch::A, &self.valuation, &self.variables);
                        }
                        index += 1;
                    } else if variable_id == watched_b.v_id() {
                        if not_watch_witness(&self.valuation, watched_a) {
                            stored_clause.update_watch(Watch::B, &self.valuation, &self.variables);
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

    pub fn literal_set_from_vec(&mut self, choices: Vec<VariableId>) {
        for v_id in choices {
            let the_literal = Literal::new(v_id, false);
            self.literal_update(the_literal, &Source::HobsonChoice);
            self.consequence_q.push_back(the_literal);
        }
    }

    fn examine_consequences_of(&mut self, literal: Literal) -> Option<ClauseKey> {
        let the_variable = self.get_variable(literal.index());

        let borrowed_occurrences = match literal.polarity() {
            true => unsafe { &mut *the_variable.negative_occurrences.get() },
            false => unsafe { &mut *the_variable.positive_occurrences.get() },
        };

        let mut index = 0;
        let mut length = borrowed_occurrences.len();

        while index < length {
            let clause_key = unsafe { *borrowed_occurrences.get_unchecked(index) };

            let stored_clause = self.stored_clauses.retreive_unchecked(clause_key);

            let watch_a = stored_clause.get_watched(Watch::A);
            let watch_b = stored_clause.get_watched(Watch::B);

            if watch_a.v_id() != literal.v_id() && watch_b.v_id() != literal.v_id() {
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
                        self.literal_update(watch_b, &Source::StoredClause(clause_key));
                        self.consequence_q.push_back(watch_b);
                    }
                    (None, Some(b)) if b == watch_b.polarity() => {}
                    (None, Some(_)) => {
                        self.literal_update(watch_a, &Source::StoredClause(clause_key));
                        self.consequence_q.push_back(watch_a);
                    }
                    (Some(a), Some(b)) if a == watch_a.polarity() || b == watch_b.polarity() => {}
                    (Some(_), Some(_)) => {
                        // clean the watch lists while clearing the q
                        self.clear_queued_consequences();
                        return Some(clause_key);
                    }
                }
            }
        }
        None
    }

    // lazy removals as implemented allow the lists to get quite messy if not kept clean
    fn clear_queued_consequences(&mut self) {
        while let Some(literal) = self.consequence_q.pop_front() {
            let occurrences = match literal.polarity() {
                true => unsafe {
                    &mut *self
                        .get_variable(literal.index())
                        .negative_occurrences
                        .get()
                },
                false => unsafe {
                    &mut *self
                        .get_variable(literal.index())
                        .positive_occurrences
                        .get()
                },
            };

            let mut index = 0;
            let mut length = occurrences.len();

            while index < length {
                let clause_key = unsafe { *occurrences.get_unchecked(index) };

                match self.stored_clauses.retreive(clause_key) {
                    Some(stored_clause) => {
                        let watch_a = stored_clause.get_watched(Watch::A);
                        let watch_b = stored_clause.get_watched(Watch::B);

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
        if reduction_allowed && self.it_is_time_to_reduce(u) {
            log::debug!(target: "forget", "Forget @r {}", self.restarts);
            self.display_stats();

            self.stored_clauses.reduce(glue_strength);
        }

        if restart_allowed && self.it_is_time_to_reduce(u) {
            self.backjump(0);
            self.restarts += 1;
            self.conflicts_since_last_forget = 0;
        }
    }

    fn process_choice(
        &mut self,
        choice_index: usize,
        last_valuation: &impl Valuation,
        polarity_lean: f64,
    ) {
        log::trace!(
            "Choice: {choice_index} @ {} with activity {}",
            self.level().index(),
            self.variables[choice_index].activity()
        );
        self.add_fresh_level();
        let choice_literal = if let Some(polarity) = last_valuation.of_index(choice_index) {
            Literal::new(choice_index as VariableId, polarity)
        } else {
            Literal::new(
                choice_index as VariableId,
                rand::thread_rng().gen_bool(polarity_lean),
            )
        };
        self.literal_update(choice_literal, &Source::Choice);
        self.consequence_q.push_back(choice_literal);
    }

    fn set_hobson(&mut self) {
        let (f, t) = hobson_choices(self.stored_clauses.clauses());
        self.literal_set_from_vec(f);
        self.literal_set_from_vec(t);
    }
}

fn not_watch_witness(valuation: &impl Valuation, literal: Literal) -> bool {
    match valuation.of_index(literal.index()) {
        Some(p) => p != literal.polarity(),
        None => true,
    }
}
