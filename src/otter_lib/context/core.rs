use rand::{seq::IteratorRandom, Rng};

use crate::{
    context::{config::Config, Context, GraphClause, ImplicationGraphNode, Status as ClauseStatus},
    io::{ContextWindow, WindowItem},
    procedures::hobson_choices,
    structures::{
        clause::stored::{Source, StoredClause},
        level::{Level, LevelIndex},
        literal::{Literal, Source as LiteralSource},
        variable::{list::VariableList, VariableId},
    },
};

macro_rules! level_mut {
    ($self:ident) => {
        unsafe {
            let index = $self.levels.len() - 1;
            $self.levels.get_unchecked_mut(index)
        }
    };
}

impl Context {
    pub fn preprocess(&mut self) {
        if self.config.preprocessing {
            self.set_hobson();
        }
    }

    #[allow(unused_labels, clippy::result_unit_err)]
    pub fn solve(&mut self) -> Result<(), ()> {
        let this_total_time = std::time::Instant::now();

        self.preprocess();
        if self.config.show_stats {
            if let Some(window) = &mut self.window {
                window.draw_window(&self.config);
            }
        }

        // store parts of config for use inside the loop
        let local_config = self.config.clone();
        let time_limit = local_config.time_limit;

        'main_loop: loop {
            self.time = this_total_time.elapsed();
            if time_limit.is_some_and(|limit| self.time > limit) {
                return Ok(());
            }

            match self.step(&local_config) {
                Ok(_) => continue 'main_loop,
                Err(_) => break 'main_loop Ok(()),
            }
        }
    }

    #[allow(unused_labels, clippy::result_unit_err)]
    pub fn step(&mut self, config: &Config) -> Result<(), ()> {
        self.iterations += 1;

        'propagation: while let Some(literal) = self.variables.get_consequence() {
            let consequence =
                self.variables
                    .propagate(literal, level_mut!(self), &mut self.clause_store, config);

            match consequence {
                Ok(_) => {}
                Err(clause_key) => match self.conflict_analysis(clause_key, config) {
                    ClauseStatus::MissedImplication(_) => continue 'propagation,
                    ClauseStatus::NoSolution(key) => {
                        self.status = ClauseStatus::NoSolution(key);
                        return Err(());
                    }
                    ClauseStatus::AssertingClause(key) => {
                        self.status = ClauseStatus::AssertingClause(key);
                        self.conflict_ceremony(config);
                        return Ok(());
                    }
                    _ => panic!("bad status after analysis"),
                },
            }
        }

        self.make_choice(config)
    }

    fn conflict_ceremony(&mut self, config: &Config) {
        self.conflicts += 1;
        self.conflicts_since_last_forget += 1;
        self.conflicts_since_last_reset += 1;

        if self.conflicts % config.decay_frequency == 0 {
            self.variables.multiply_activity(config.decay_factor);
        }

        self.reductions_and_restarts(config);
    }

    #[allow(clippy::result_unit_err)]
    pub fn make_choice(&mut self, config: &Config) -> Result<(), ()> {
        match self.get_unassigned(config.random_choice_frequency) {
            Some(choice_index) => {
                self.process_choice(choice_index, config.polarity_lean);
                self.status = ClauseStatus::ChoiceMade;
                Ok(())
            }
            None => {
                self.status = ClauseStatus::AllAssigned;
                Err(())
            }
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
                self.clause_store.reduce(config.glue_strength);
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
            let choice_variable = self.variables.get_unsafe(index);

            match choice_variable.previous_polarity() {
                Some(polarity) => Literal::new(index as VariableId, polarity),
                None => Literal::new(
                    index as VariableId,
                    rand::thread_rng().gen_bool(polarity_lean),
                ),
            }
        };
        match self.variables.set_value(
            choice_literal,
            unsafe { self.levels.get_unchecked_mut(level_index) },
            LiteralSource::Choice,
        ) {
            Ok(_) => {}
            Err(e) => panic!("failed to update on choice: {e:?}"),
        };
        self.variables.push_back_consequence(choice_literal);
    }

    fn set_hobson(&mut self) {
        let (f, t) = hobson_choices(self.clause_store.clauses());

        for v_id in f.into_iter().chain(t) {
            let the_literal = Literal::new(v_id, false);
            match self.variables.set_value(
                the_literal,
                unsafe { self.levels.get_unchecked_mut(0) },
                LiteralSource::HobsonChoice,
            ) {
                Ok(_) => {}
                Err(e) => panic!("issue on hobson update: {e:?}"),
            };
            self.variables.push_back_consequence(the_literal);
        }
    }

    pub fn add_fresh_level(&mut self) -> LevelIndex {
        let index = self.levels.len();
        self.levels.push(Level::new(index));
        index
    }

    pub fn level(&self) -> &Level {
        let index = self.levels.len() - 1;
        unsafe { self.levels.get_unchecked(index) }
    }

    pub fn level_zero(&self) -> &Level {
        unsafe { self.levels.get_unchecked(0) }
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

        let clause_key = self.clause_store.insert(src, clause, &self.variables);
        let the_clause = self.clause_store.retreive_mut(clause_key);
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
            self.update_stats(self.window.as_ref().expect("no window"));
            self.window.as_ref().unwrap().flush();
        }

        match self.status {
            ClauseStatus::AllAssigned => {
                println!("s SATISFIABLE");
                if self.config.show_valuation {
                    println!("v {}", self.variables().as_display_string());
                }
                // std::process::exit(10);
            }
            ClauseStatus::NoSolution(clause_key) => {
                println!("s UNSATISFIABLE");
                if self.config.show_core {
                    self.display_core(clause_key);
                }
                // std::process::exit(20);
            }
            _ => {
                if let Some(limit) = self.config.time_limit {
                    if self.config.show_stats && self.time > limit {
                        println!("c TIME LIMIT EXCEEDED");
                    }
                }
                println!("s UNKNOWN");
                // std::process::exit(30);
            }
        }
    }

    pub fn clause_count(&self) -> usize {
        self.clause_store.formula_count() + self.clause_store.learned_count()
    }
}
