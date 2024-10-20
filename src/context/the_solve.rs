use crate::{
    context::{config::Config, Context, Result, Status},
    procedures::hobson_choices,
    structures::{
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

            'literal_consequences: while let Some(literal) = self.variables.get_consequence() {
                let this_level_index = self.level().index();
                let this_level = self.levels.get_mut(this_level_index).expect("lost level");

                let consequences = self.variables.propagate(
                    literal,
                    this_level,
                    &mut self.stored_clauses,
                    &local_config,
                );

                match consequences {
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
                                    self.variables.multiply_activity(local_config.decay_factor);
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

    pub fn literal_set_from_vec(&mut self, choices: Vec<VariableId>) {
        for v_id in choices {
            let the_literal = Literal::new(v_id, false);
            self.literal_update(the_literal, 0, Source::HobsonChoice);
            self.variables.push_back_consequence(the_literal);
        }
    }

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
