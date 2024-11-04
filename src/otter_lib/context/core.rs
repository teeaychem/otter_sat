use rand::{seq::IteratorRandom, Rng};

use crate::{
    config::{self, Config},
    context::{level::LevelIndex, store::ClauseKey, Context, Report, SolveStatus},
    structures::{
        clause::{
            stored::{ClauseSource, StoredClause},
            Clause,
        },
        literal::{Literal, LiteralSource},
        variable::{
            core::propagate_literal, delegate::queue_consequence, list::VariableList, VariableId,
        },
    },
};

#[derive(Debug, Clone, Copy)]
pub enum ContextIssue {
    EmptyClause,
}

pub enum StepInfo {
    Conflict(ClauseKey),
    QueueConflict(ClauseKey),
    QueueProof(ClauseKey),
    ChoicesExhausted,
}

impl Context {
    #[allow(unused_labels, clippy::result_unit_err)]
    pub fn solve(&mut self) -> Result<Report, ()> {
        let this_total_time = std::time::Instant::now();

        self.preprocess();

        if self.clause_store.clause_count() == 0 {
            self.status = SolveStatus::NoClauses;
            return Ok(Report::Satisfiable);
        }

        if self.config.show_stats {
            if let Some(window) = &mut self.window {
                window.draw_window(&self.config);
            }
        }

        let local_config = self.config.clone();
        let time_limit = local_config.time_limit;

        'main_loop: loop {
            self.counters.time = this_total_time.elapsed();
            if time_limit.is_some_and(|limit| self.counters.time > limit) {
                return Ok(self.report());
            }

            match self.step(&local_config) {
                Ok(_) => continue 'main_loop,
                Err(_) => {
                    break 'main_loop Ok(self.report());
                }
            }
        }
    }

    #[allow(unused_labels, clippy::result_unit_err)]
    pub fn step(&mut self, config: &Config) -> Result<(), StepInfo> {
        self.counters.iterations += 1;

        'search: while let Some((literal, _source, _)) = self.variables.get_consequence() {
            let consequence = propagate_literal(
                literal,
                &mut self.variables,
                &mut self.clause_store,
                self.levels.top_mut(),
            );

            match consequence {
                Ok(()) => {}
                Err(key) => {
                    let analysis_result = self.conflict_analysis(key, config);
                    match analysis_result {
                        Ok(analysis_result) => {
                            use super::analysis::AnalysisResult::*;
                            match analysis_result {
                                MissedImplication(key, literal) => {
                                    self.status = SolveStatus::MissedImplication(key);

                                    let the_clause = self.clause_store.get(key);
                                    let missed_level =
                                        self.backjump_level(the_clause.literal_slice());
                                    self.backjump(missed_level);
                                    match queue_consequence(
                                        &mut self.variables,
                                        literal,
                                        LiteralSource::Missed(key),
                                        self.levels.top_mut(),
                                    ) {
                                        Ok(()) => {}
                                        Err(key) => {
                                            return Err(StepInfo::QueueConflict(key));
                                        }
                                    };

                                    continue 'search;
                                }
                                FundamentalConflict(key) | QueueConflict(key) => {
                                    self.status = SolveStatus::NoSolution(key);

                                    return Err(StepInfo::Conflict(key));
                                }
                                Proof(key, literal) => {
                                    self.status = SolveStatus::Proof(key);

                                    self.backjump(0);
                                    match queue_consequence(
                                        &mut self.variables,
                                        literal,
                                        LiteralSource::Resolution(key),
                                        self.levels.top_mut(),
                                    ) {
                                        Ok(()) => {}
                                        Err(key) => return Err(StepInfo::QueueProof(key)),
                                    }
                                }

                                AssertingClause(key, literal) => {
                                    self.status = SolveStatus::AssertingClause(key);

                                    let the_clause = self.clause_store.get(key);

                                    let backjump_level_index =
                                        self.backjump_level(the_clause.literal_slice());
                                    self.backjump(backjump_level_index);

                                    match queue_consequence(
                                        &mut self.variables,
                                        literal,
                                        LiteralSource::Analysis(key),
                                        self.levels.top_mut(),
                                    ) {
                                        Ok(()) => {}
                                        Err(key) => return Err(StepInfo::QueueConflict(key)),
                                    }

                                    self.conflict_ceremony(config);
                                    return Ok(());
                                }
                            }
                        }
                        Err(_issue) => {
                            log::error!(target: crate::log::targets::STEP, "Conflict analysis failed.");
                            panic!("Analysis failed")
                        }
                    }
                }
            }
        }

        self.make_choice(config)
    }

    fn conflict_ceremony(&mut self, config: &Config) {
        self.counters.conflicts += 1;
        self.counters.conflicts_since_last_forget += 1;
        self.counters.conflicts_since_last_reset += 1;

        if self.it_is_time_to_restart(config.luby_constant) {
            if let Some(window) = &self.window {
                window.update_counters(&self.counters);
                window.flush();
            }

            if config.restarts_allowed {
                self.backjump(0);
                self.counters.restarts += 1;
                self.counters.conflicts_since_last_forget = 0;
            }

            if config.reduction_allowed
                && ((self.counters.restarts % config.reduction_interval) == 0)
            {
                log::debug!(target: crate::log::targets::REDUCTION, "Forget @r {}", self.counters.restarts);
                self.clause_store.reduce();
            }
        }
    }

    #[allow(clippy::result_unit_err)]
    pub fn make_choice(&mut self, config: &Config) -> Result<(), StepInfo> {
        self.levels.get_fresh();
        match self.get_unassigned(config.random_choice_frequency) {
            Some(choice_index) => {
                self.process_choice(choice_index, config.polarity_lean);
                self.counters.decisions += 1;
                self.status = SolveStatus::ChoiceMade;
                Ok(())
            }
            None => {
                self.status = SolveStatus::AllAssigned;
                Err(StepInfo::ChoicesExhausted)
            }
        }
    }

    fn process_choice(&mut self, index: usize, polarity_lean: config::PolarityLean) {
        log::trace!(target: crate::log::targets::STEP,
            "Choice of {index} at level {} with activity {}",
            self.levels.top().index(),
            self.variables.activity_of(index)
        );
        let choice_literal = {
            let choice_variable = self.variables.get_unsafe(index);

            match choice_variable.previous_value() {
                Some(polarity) => Literal::new(index as VariableId, polarity),
                None => Literal::new(index as VariableId, self.rng.gen_bool(polarity_lean)),
            }
        };
        match queue_consequence(
            &mut self.variables,
            choice_literal,
            LiteralSource::Choice,
            self.levels.top_mut(),
        ) {
            Ok(()) => {}
            Err(_) => panic!("could not set choice"),
        };
    }

    pub fn proven_literals(&self) -> impl Iterator<Item = &Literal> {
        self.levels
            .get(0)
            .observations()
            .iter()
            .map(|(_, literal)| literal)
    }

    pub fn variables(&self) -> &impl VariableList {
        &self.variables
    }

    pub fn get_unassigned(
        &mut self,
        random_choice_frequency: config::RandomChoiceFrequency,
    ) -> Option<usize> {
        match self.rng.gen_bool(random_choice_frequency) {
            true => self
                .variables
                .iter()
                .filter(|variable| variable.value().is_none())
                .choose(&mut self.rng)
                .map(|variable| variable.index()),
            false => {
                while let Some(index) = self.variables.heap_pop_most_active() {
                    let the_variable = self.variables.get_unsafe(index);
                    if self.variables.value_of(the_variable.index()).is_none() {
                        return Some(the_variable.index());
                    }
                }
                self.variables
                    .iter()
                    .filter(|variable| variable.value().is_none())
                    .map(|x| x.index())
                    .next()
            }
        }
    }

    /// Stores a clause with an automatically generated id.
    /// In order to use the clause the watch literals of the struct must be initialised.
    pub fn store_clause(
        &mut self,
        clause: Vec<Literal>,
        subsumed: Vec<Literal>,
        src: ClauseSource,
        resolution_keys: Option<Vec<ClauseKey>>,
    ) -> Result<&StoredClause, ContextIssue> {
        if clause.is_empty() {
            return Err(ContextIssue::EmptyClause);
        }
        assert!(clause.len() > 1, "Attempt to add a short clause");

        let clause_key =
            self.clause_store
                .insert(src, clause, subsumed, &mut self.variables, resolution_keys);
        let the_clause = self.clause_store.get_mut(clause_key);
        Ok(the_clause)
    }

    pub fn backjump(&mut self, to: LevelIndex) {
        log::trace!(target: crate::log::targets::STEP, "Backjump from {} to {}", self.levels.top().index(), to);

        for _ in 0..(self.levels.top().index() - to) {
            for literal in self.levels.pop().expect("Lost level").literals() {
                log::trace!(target: crate::log::targets::STEP, "Noneset: {}", literal.index());
                self.variables.retract_valuation(literal.index());
                self.variables.heap_push(literal.index());
            }
        }
        self.variables.clear_consequences(to);
    }

    pub fn print_status(&self) {
        if self.config.show_stats {
            if let Some(window) = &self.window {
                window.update_counters(&self.counters);
                window.flush();
            }
        }

        match self.status {
            SolveStatus::AllAssigned => {
                println!("s SATISFIABLE");
                if self.config.show_valuation {
                    print!("v");
                    for v in self.variables().slice() {
                        match v.value() {
                            Some(true) => print!(" {}", self.variables.external_name(v.index())),
                            Some(false) => print!(" -{}", self.variables.external_name(v.index())),
                            None => panic!("variables were not all assigned"),
                        }
                    }
                    println!();
                }
                // std::process::exit(10);
            }
            SolveStatus::NoSolution(clause_key) => {
                println!("s UNSATISFIABLE");
                if self.config.show_core {
                    self.display_core(clause_key);
                }
                // std::process::exit(20);
            }
            SolveStatus::NoClauses => {
                println!("c The formula contains no clause and so is interpreted as ⊤");
                println!("s SATISFIABLE");
            }
            _ => {
                if let Some(limit) = self.config.time_limit {
                    if self.config.show_stats && self.counters.time > limit {
                        println!("c TIME LIMIT EXCEEDED");
                    }
                }
                println!("s UNKNOWN");
                // std::process::exit(30);
            }
        }
    }

    pub fn clause_count(&self) -> usize {
        self.clause_store.clause_count()
    }

    pub fn it_is_time_to_restart(&self, u: config::LubyConstant) -> bool {
        use crate::procedures::luby;
        self.counters.conflicts_since_last_forget >= u.wrapping_mul(luby(self.counters.restarts))
    }

    pub fn report(&self) -> Report {
        match self.status {
            SolveStatus::AllAssigned => Report::Satisfiable,
            SolveStatus::NoSolution(_) => Report::Unsatisfiable,
            SolveStatus::NoClauses => Report::Satisfiable,
            _ => Report::Unknown,
        }
    }

    pub fn valuation_string(&self) -> String {
        self.variables
            .slice()
            .iter()
            .enumerate()
            .filter_map(|(i, v)| match v.value() {
                None => None,
                Some(true) => Some(self.variables.external_name(i).to_string()),
                Some(false) => Some(format!("-{}", self.variables.external_name(i))),
            })
            .collect::<Vec<_>>()
            .join(" ")
    }

    pub fn internal_valuation_string(&self) -> String {
        let mut v = self
            .variables
            .slice()
            .iter()
            .enumerate()
            .filter_map(|(i, v)| match v.value() {
                None => None,
                Some(true) => Some(i as isize),
                Some(false) => Some(-(i as isize)),
            })
            .collect::<Vec<_>>();
        v.sort_unstable();
        v.iter()
            .map(|v| v.to_string())
            .collect::<Vec<_>>()
            .join(" ")
    }

    pub fn print_valuation(&self) {
        println!("v {:?}", self.valuation_string());
    }
}
