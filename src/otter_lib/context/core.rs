use rand::{seq::IteratorRandom, Rng};

use crate::{
    context::{config::Config, Context, GraphClause, ImplicationGraphNode, Status},
    io::{ContextWindow, WindowItem},
    procedures::hobson_choices,
    structures::{
        clause::stored::{Source, StoredClause},
        level::{Level, LevelIndex},
        literal::{Literal, Source as LiteralSource},
        variable::{list::VariableList, VariableId},
    },
};

impl Context {
    #[allow(unused_labels, clippy::result_unit_err)]
    pub fn solve(&mut self) -> Result<(), ()> {
        let this_total_time = std::time::Instant::now();

        if self.config.hobson_choices {
            self.set_hobson();
        }

        // store parts of config for use inside the loop
        let local_config = self.config.clone();
        let time_limit = local_config.time_limit;

        'main_loop: loop {
            self.time = this_total_time.elapsed();
            if time_limit.is_some_and(|limit| self.time > limit) {
                return Ok(());
            }
            let step = self.step(&local_config);
            match step {
                Ok(_) => continue 'main_loop,
                Err(_) => break 'main_loop Ok(()),
            }
        }
    }

    #[allow(unused_labels, clippy::result_unit_err)]
    pub fn step(&mut self, local_config: &Config) -> Result<(), ()> {
        self.iterations += 1;

        'literal_consequences: while let Some(literal) = self.variables.get_consequence() {
            let this_level_index = self.level().index();
            let this_level = self.levels.get_mut(this_level_index).expect("lost level");

            let consequences = self.variables.propagate(
                literal,
                this_level,
                &mut self.stored_clauses,
                local_config,
            );

            match consequences {
                Ok(_) => {}
                Err(conflict_key) => {
                    self.status = self.conflict_analysis(conflict_key, local_config);

                    match self.status {
                        Status::NoSolution(_) => return Ok(()),
                        Status::MissedImplication(_) => return Ok(()),
                        Status::AssertingClause(_) => {
                            for variable in self.variables.slice().iter() {
                                self.last_valuation[variable.index()] = variable.polarity();
                            }

                            self.conflicts += 1;
                            self.conflicts_since_last_forget += 1;
                            self.conflicts_since_last_reset += 1;

                            if self.conflicts % local_config.decay_frequency == 0 {
                                self.variables.multiply_activity(local_config.decay_factor);
                            }

                            self.reductions_and_restarts(local_config);
                            return Ok(());
                        }
                        _ => panic!("bad status after analysis"),
                    }
                }
            }
        }

        match self.get_unassigned(local_config.random_choice_frequency) {
            Some(choice_index) => {
                self.process_choice(choice_index, local_config.polarity_lean);
                Ok(())
            }
            None => Err(()),
        }
    }

    /*
    let level_index = match source {
            Source::Choice | Source::Clause(_) => &self.levels.len() - 1,
            Source::Assumption | Source::HobsonChoice | Source::Resolution(_) => 0,
            };
     */
    pub fn literal_update(
        &mut self,
        literal: Literal,
        level_index: LevelIndex,
        source: LiteralSource,
    ) {
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
            self.literal_update(the_literal, 0, LiteralSource::HobsonChoice);
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

    fn process_choice(&mut self, index: usize, polarity_lean: f64) {
        log::trace!(
            "Choice: {index} @ {} with activity {}",
            self.level().index(),
            self.variables.get_unsafe(index).activity()
        );
        let level_index = self.add_fresh_level();
        let choice_literal = {
            let id = index as VariableId;
            match &self.last_valuation[index] {
                Some(polarity) => Literal::new(id, *polarity),
                None => Literal::new(id, rand::thread_rng().gen_bool(polarity_lean)),
            }
        };
        self.literal_update(choice_literal, level_index, LiteralSource::Choice);
        self.variables.push_back_consequence(choice_literal);
    }

    fn set_hobson(&mut self) {
        let (f, t) = hobson_choices(self.stored_clauses.clauses());
        self.literal_set_from_vec(f);
        self.literal_set_from_vec(t);
    }

    pub fn add_fresh_level(&mut self) -> LevelIndex {
        let index = self.levels.len();
        let the_level = Level::new(index);
        self.levels.push(the_level);
        index
    }

    pub fn level(&self) -> &Level {
        let index = self.levels.len() - 1;
        &self.levels[index]
    }

    pub fn level_zero(&self) -> &Level {
        &self.levels[0]
    }

    pub fn variables(&self) -> &impl VariableList {
        &self.variables
    }

    pub fn get_unassigned(&self, random_choice_frequency: f64) -> Option<usize> {
        match rand::thread_rng().gen_bool(random_choice_frequency) {
            true => self
                .variables
                .iter()
                .filter(|variable| variable.polarity().is_none())
                .choose(&mut rand::thread_rng())
                .map(|variable| variable.index()),
            false => self
                .variables
                .iter()
                .enumerate()
                .filter(|(_, variable)| variable.polarity().is_none())
                .map(|(index, _)| (index, self.variables[index].activity()))
                .max_by(|(_, activity_a), (_, activity_b)| activity_a.total_cmp(activity_b))
                .map(|(index, _)| index),
        }
    }

    /// Stores a clause with an automatically generated id.
    /// In order to use the clause the watch literals of the struct must be initialised.
    pub fn store_clause(&mut self, clause: Vec<Literal>, src: Source) -> &StoredClause {
        assert!(!clause.is_empty(), "Attempt to add an empty clause");

        let clause_key = self.stored_clauses.insert(src, clause, &self.variables);
        let the_clause = self.stored_clauses.retreive_mut(clause_key);
        let node_index = self
            .implication_graph
            .add_node(ImplicationGraphNode::Clause(GraphClause {
                clause_id: the_clause.id(),
                key: the_clause.key(),
            }));
        the_clause.add_node_index(node_index);
        the_clause
    }

    pub fn backjump(&mut self, to: LevelIndex) {
        log::trace!("Backjump from {} to {}", self.level().index(), to);

        for _ in 0..(self.level().index() - to) {
            for literal in self.levels.pop().expect("Lost level").literals() {
                log::trace!("Noneset: {}", literal.index());
                self.variables.retract_valuation(literal.index());
            }
        }
    }

    pub fn update_stats(&self, window: &ContextWindow) {
        window.update_item(WindowItem::Iterations, self.iterations);
        window.update_item(WindowItem::Conflicts, self.conflicts);
        window.update_item(
            WindowItem::Ratio,
            self.conflicts as f32 / self.iterations as f32,
        );
        window.update_item(WindowItem::Time, format!("{:.2?}", self.time));
    }

    pub fn print_status(&self) {
        if self.config.show_stats {
            self.update_stats(self.window.as_ref().unwrap());
            self.window.as_ref().unwrap().flush();
        }

        match self.status {
            Status::AllAssigned => {
                println!("s SATISFIABLE");
                if self.config.show_valuation {
                    println!("v {}", self.variables().as_display_string());
                }
                std::process::exit(10);
            }
            Status::NoSolution(clause_key) => {
                println!("s UNSATISFIABLE");
                if self.config.show_core {
                    self.display_core(clause_key);
                }
                std::process::exit(20);
            }
            _ => {
                if let Some(limit) = self.config.time_limit {
                    if self.config.show_stats && self.time > limit {
                        println!("c TIME LIMIT EXCEEDED");
                    }
                }
                println!("s UNKNOWN");
                std::process::exit(30);
            }
        }
    }
}
