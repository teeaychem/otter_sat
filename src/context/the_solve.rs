use crate::{
    context::{config::Config, Context, Result, Status},
    procedures::hobson_choices,
    structures::{
        clause::stored::Watch,
        level::LevelIndex,
        literal::{Literal, Source},
        variable::{list::VariableList, VariableId},
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
        let local_config = self.config.clone();
        let time_limit = local_config.time_limit;

        'main_loop: loop {
            self.iterations += 1;

            self.time = this_total_time.elapsed();
            if time_limit.is_some_and(|limit| self.time > limit) {
                return Result::Unknown;
            }

            let current_index = self.level().index();

            'literal_consequences: while let Some(literal) = self.variables.pop_front_consequence()
            {
                self.update_watches(literal);
                let consequence = self.variables.examine_consequences(
                    literal,
                    self.levels.get_mut(current_index).expect("missing level"),
                    &self.stored_clauses,
                );
                match consequence {
                    Ok(_) => {}
                    Err(conflict_key) => {
                        let analysis = self.conflict_analysis(conflict_key, &local_config);

                        match analysis {
                            Status::NoSolution => return Result::Unsatisfiable(conflict_key),
                            Status::MissedImplication => continue 'main_loop,
                            Status::AssertingClause => {
                                for variable in self.variables.slice().iter() {
                                    last_valuation[variable.index()] = variable.polarity();
                                }

                                self.conflicts += 1;
                                self.conflicts_since_last_forget += 1;
                                self.conflicts_since_last_reset += 1;

                                if self.conflicts % local_config.decay_frequency == 0 {
                                    for variable in self.variables.slice().iter() {
                                        variable.multiply_activity(local_config.decay_factor);
                                    }
                                }

                                self.reductions_and_restarts(&local_config);
                                continue 'main_loop;
                            }
                        }
                    }
                }
            }

            match self.get_unassigned(local_config.random_choice_frequency) {
                Some(choice_index) => {
                    self.process_choice(choice_index, &last_valuation, local_config.polarity_lean);
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
            self.variables.push_back_consequence(the_literal);
        }
    }

    /*
    todo: transfer to valuation store
    return the new_observations and import these to the context there
    though, use a buffer of variable size to avoid mallocs
    this then is a step toward a better handling of unsafe as the borrow from valuation store while updating other valuations in the store is clearer
     */

    fn reductions_and_restarts(&mut self, config: &Config) {
        if self.it_is_time_to_reduce(config.luby_constant) {
            if let Some(window) = &self.window {
                self.update_stats(window);
                window.flush();
            }

            if config.reduction_allowed {
                log::debug!(target: "forget", "Forget @r {}", self.restarts);
                self.stored_clauses.reduce(config.glue_strength);
            }

            if config.restarts_allowed {
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
        self.variables.push_back_consequence(choice_literal);
    }

    fn set_hobson(&mut self) {
        let (f, t) = hobson_choices(self.stored_clauses.clauses());
        self.literal_set_from_vec(f);
        self.literal_set_from_vec(t);
    }
}
