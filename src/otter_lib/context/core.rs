use crate::{
    context::{stores::ClauseKey, Context, Report, SolveStatus},
    structures::{
        clause::stored::{ClauseSource, StoredClause},
        literal::Literal,
        variable::list::VariableList,
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
    Backfall,
    AnalysisFailure,
    ChoiceFailure,
}

#[derive(Debug)]
pub enum ContextFailure {}

impl Context {
    pub fn proven_literals(&self) -> impl Iterator<Item = &Literal> {
        self.levels
            .get(0)
            .observations()
            .iter()
            .map(|(_, literal)| literal)
    }

    pub fn variable_count(&self) -> usize {
        self.variables.len()
    }

    pub fn clause_count(&self) -> usize {
        self.clause_store.clause_count()
    }

    /// Stores a clause with an automatically generated id.
    /// In order to use the clause the watch literals of the struct must be initialised.
    pub fn store_clause(
        &mut self,
        clause: Vec<Literal>,
        subsumed: Vec<Literal>,
        source: ClauseSource,
        resolution_keys: Option<Vec<ClauseKey>>,
    ) -> Result<&StoredClause, ContextIssue> {
        if clause.is_empty() {
            return Err(ContextIssue::EmptyClause);
        }
        assert!(clause.len() > 1, "Attempt to add a short clause");

        let clause_key = self.clause_store.insert(
            source,
            clause,
            subsumed,
            &mut self.variables,
            resolution_keys,
        );
        let the_clause = self.clause_store.get_mut(clause_key);
        Ok(the_clause)
    }

    pub fn print_status(&self) {
        if self.config.show_stats {
            if let Some(window) = &self.window {
                window.update_counters(&self.counters);
                window.flush();
            }
        }

        match self.status {
            SolveStatus::FullValuation => {
                println!("s SATISFIABLE");
                if self.config.show_valuation {
                    println!("v {}", self.valuation_string());
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
                if self.config.verbosity > 0 {
                    println!("c The formula contains no clause and so is interpreted as ⊤");
                }
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

    pub fn report(&self) -> Report {
        match self.status {
            SolveStatus::FullValuation => Report::Satisfiable,
            SolveStatus::NoClauses => Report::Satisfiable,
            SolveStatus::NoSolution(_) => Report::Unsatisfiable,
            _ => Report::Unknown,
        }
    }

    pub fn valuation_string(&self) -> String {
        self.variables
            .slice()
            .iter()
            .filter_map(|v| match v.value() {
                None => None,
                Some(true) => Some(self.variables.external_name(v.index()).to_string()),
                Some(false) => Some(format!("-{}", self.variables.external_name(v.index()))),
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
