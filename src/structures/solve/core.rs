use crate::structures::{
    solve::{solves::literal_update, ExplorationPriority, SolveConfig},
    stored_clause::initialise_watches_for,
    Clause, ClauseId, ClauseSource, ClauseStatus, Formula, Level, LevelIndex, Literal,
    LiteralError, LiteralSource, StoredClause, Valuation, ValuationVec, Variable, VariableId,
    WatchStatus,
};

use std::sync::atomic::{AtomicUsize, Ordering as AtomicOrdering};

use std::collections::VecDeque;
use std::rc::Rc;

pub struct SolveStatus {
    pub implications: Vec<(Rc<StoredClause>, Literal)>,
    pub conflict_clauses: Vec<Rc<StoredClause>>,
}

impl SolveStatus {
    pub fn new() -> Self {
        SolveStatus {
            implications: vec![],
            conflict_clauses: vec![],
        }
    }
}

#[derive(Debug)]
pub struct Solve<'formula> {
    _formula: &'formula Formula,
    pub conflicts: usize,
    pub conflcits_since_last_forget: usize,
    pub forgets: usize,
    pub variables: Vec<Variable>,
    pub valuation: Vec<Option<bool>>,
    pub levels: Vec<Level>,
    pub formula_clauses: Vec<Rc<StoredClause>>,
    pub learnt_clauses: Vec<Rc<StoredClause>>,
    pub watch_q: VecDeque<Literal>,
    pub config: SolveConfig,
}

#[derive(Debug, PartialEq)]
pub enum SolveOk {
    AssertingClause,
    Deduction(Literal),
    Backtracked,
}

#[derive(Debug)]
pub enum SolveError {
    Literal(LiteralError),
    // Clause(ClauseError),
    OutOfBounds,
    UnsatClause(Rc<StoredClause>),
    Conflict(Rc<StoredClause>, Literal),
    NoSolution,
}

impl Solve<'_> {
    pub fn from_formula(formula: &Formula, config: SolveConfig) -> Solve {
        let mut the_solve = Solve {
            _formula: formula,
            conflicts: 0,
            conflcits_since_last_forget: 0,
            forgets: 0,
            variables: formula.vars().to_vec(),
            watch_q: VecDeque::with_capacity(formula.vars().len() / 4), // I expect this to be mostly empty
            valuation: Vec::<Option<bool>>::new_for_variables(formula.vars().len()),
            levels: vec![Level::new(0)],
            formula_clauses: Vec::new(),
            learnt_clauses: Vec::new(),
            config,
        };

        let initial_valuation = the_solve.valuation.clone();

        formula
            .clauses()
            .for_each(|formula_clause| match formula_clause.length() {
                0 => {
                    panic!("c The formula contains a zero-length clause");
                }
                _ => {
                    let clause =
                        the_solve.store_clause(formula_clause.as_vec(), ClauseSource::Formula);
                    initialise_watches_for(&clause, &initial_valuation, &mut the_solve.variables);
                }
            });

        the_solve
    }

    pub fn valuation_at(&self, level_index: LevelIndex) -> ValuationVec {
        let mut valuation = ValuationVec::new_for_variables(self.valuation.len());
        (0..=level_index).for_each(|i| {
            self.levels[i].literals().for_each(|l| {
                let _ = valuation.update_value(l);
            })
        });
        valuation
    }

    pub fn stored_clauses(&self) -> impl Iterator<Item = &Rc<StoredClause>> {
        self.formula_clauses.iter().chain(&self.learnt_clauses)
    }

    pub fn clauses(&self) -> impl Iterator<Item = &impl Clause> {
        self.stored_clauses()
            .map(|stored_clause| stored_clause.clause())
    }

    pub fn fresh_clause_id() -> ClauseId {
        static COUNTER: AtomicUsize = AtomicUsize::new(1);
        COUNTER.fetch_add(1, AtomicOrdering::Relaxed) as ClauseId
    }

    pub fn get_unassigned_id(&self, solve: &Solve) -> Option<VariableId> {
        solve
            .variables
            .iter()
            .find(|&v| self.valuation.of_v_id(v.id()).is_none())
            .map(|found| found.id())
    }

    pub fn var_by_id(&self, id: VariableId) -> Option<&Variable> {
        self.variables.get(id)
    }

    pub fn set_from_lists(&mut self, the_choices: (Vec<VariableId>, Vec<VariableId>)) {
        the_choices.0.iter().for_each(|&v_id| {
            let the_literal = Literal::new(v_id, false);
            match literal_update(
                the_literal,
                LiteralSource::HobsonChoice,
                &mut self.levels,
                &mut self.variables,
                &mut self.valuation,
            ) {
                WatchStatus::Implication => match self.config.conflict_priority {
                    ExplorationPriority::Implication => self.watch_q.push_front(the_literal),
                    _ => self.watch_q.push_back(the_literal),
                },
                WatchStatus::Conflict => match self.config.conflict_priority {
                    ExplorationPriority::Conflict => self.watch_q.push_front(the_literal),
                    _ => self.watch_q.push_back(the_literal),
                },
                _ => {}
            };
        });
        the_choices.1.iter().for_each(|&v_id| {
            let the_literal = Literal::new(v_id, true);
            match literal_update(
                the_literal,
                LiteralSource::HobsonChoice,
                &mut self.levels,
                &mut self.variables,
                &mut self.valuation,
            ) {
                WatchStatus::Implication => match self.config.conflict_priority {
                    ExplorationPriority::Implication => self.watch_q.push_front(the_literal),
                    _ => self.watch_q.push_back(the_literal),
                },
                WatchStatus::Conflict => match self.config.conflict_priority {
                    ExplorationPriority::Conflict => self.watch_q.push_front(the_literal),
                    _ => self.watch_q.push_back(the_literal),
                },
                _ => {}
            };
        });
    }

    pub fn select_conflict(&self, clauses: &[Rc<StoredClause>]) -> Option<Rc<StoredClause>> {
        clauses.first().cloned()
    }
}

impl Solve<'_> {
    pub fn examine_clauses<'a>(
        &'a self,
        val: &'a impl Valuation,
        clauses: impl Iterator<Item = Rc<StoredClause>> + 'a,
    ) -> impl Iterator<Item = (Rc<StoredClause>, ClauseStatus)> + 'a {
        clauses.flat_map(|stored_clause| match stored_clause.watch_choices(val) {
            ClauseStatus::Conflict => Some((stored_clause.clone(), ClauseStatus::Conflict)),
            ClauseStatus::Entails(the_literal) => {
                Some((stored_clause.clone(), ClauseStatus::Entails(the_literal)))
            }
            _ => None,
        })
    }
}

impl Solve<'_> {
    pub fn notice_conflict(&mut self, stored_clauses: &Rc<StoredClause>) {
        self.conflicts += 1;
        self.conflcits_since_last_forget += 1;
        if self.conflicts % 256 == 0 {
            for variable in &mut self.variables {
                variable.divide_activity(2.0)
            }
        }

        for literal in stored_clauses.variables() {
            self.variables[literal].add_activity(1.0);
        }
    }

    pub fn most_active_none(&self, val: &impl Valuation) -> Option<VariableId> {
        val.values()
            .enumerate()
            .filter(|(_, v)| v.is_none())
            .map(|(i, _)| (i, self.variables[i].activity()))
            .max_by(|a, b| a.1.total_cmp(&b.1))
            .map(|(a, _)| a)
    }

    pub fn time_to_reduce(&self) -> bool {
        self.conflcits_since_last_forget > (1000 + 300 * self.forgets)
    }
}

impl std::fmt::Display for Solve<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let _ = writeln!(f, "Valuation: {}", self.valuation.as_display_string(self));
        let _ = write!(f, "More to be added…");
        Ok(())
    }
}
