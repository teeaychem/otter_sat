use crate::structures::{
    clause::{
        stored_clause::{initialise_watches_for, ClauseSource, ClauseStatus, StoredClause},
        Clause, ClauseId,
    },
    formula::Formula,
    level::{Level, LevelIndex},
    literal::{Literal, LiteralSource},
    solve::the_solve::literal_update,
    solve::Solve,
    valuation::{Valuation, ValuationVec},
    variable::{Variable, VariableId},
};

use std::sync::atomic::{AtomicUsize, Ordering as AtomicOrdering};

use std::collections::VecDeque;
use std::rc::Rc;

impl Solve<'_> {
    pub fn from_formula(formula: &Formula) -> Solve {
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
            literal_update(
                the_literal,
                LiteralSource::HobsonChoice,
                &mut self.levels,
                &mut self.variables,
                &mut self.valuation,
            );
            self.watch_q.push_back(the_literal);
        });
        the_choices.1.iter().for_each(|&v_id| {
            let the_literal = Literal::new(v_id, true);
            literal_update(
                the_literal,
                LiteralSource::HobsonChoice,
                &mut self.levels,
                &mut self.variables,
                &mut self.valuation,
            );
            self.watch_q.push_back(the_literal);
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
        // if self.conflicts % 1024 == 0 {
        // for variable in &mut self.variables {
        //     variable.divide_activity(2.0)
        // }
        // }

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

    pub fn is_it_time_to_reduce(&self) -> bool {
        self.conflcits_since_last_forget > (1000 + 200 * self.forgets)
    }
}

impl std::fmt::Display for Solve<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let _ = writeln!(f, "Valuation: {}", self.valuation.as_display_string(self));
        let _ = write!(f, "More to be added…");
        Ok(())
    }
}

impl<'borrow, 'solve> Solve<'solve> {
    /// Stores a clause with an automatically generated id.
    /// Note: In order to use the clause the watch literals of the struct must be initialised.
    pub fn store_clause(
        &'borrow mut self,
        clause: impl Clause,
        src: ClauseSource,
    ) -> Rc<StoredClause> {
        match clause.length() {
            0 => panic!("Attempt to add an empty clause"),
            _ => match &src {
                ClauseSource::Formula => {
                    let stored_clause =
                        StoredClause::new_from(Solve::fresh_clause_id(), clause, src);

                    for literal in stored_clause.literals() {
                        self.variables[literal.v_id]
                            .note_occurence(&stored_clause, literal.polarity);
                    }

                    self.formula_clauses.push(stored_clause.clone());
                    stored_clause
                }
                ClauseSource::Resolution(_) => {
                    log::trace!("Learning clause {}", clause.as_string());
                    let stored_clause =
                        StoredClause::new_from(Solve::fresh_clause_id(), clause, src);

                    for variable in &mut self.variables {
                        variable.divide_activity(1.2)
                    }
                    for literal in stored_clause.literals() {
                        self.variables[literal.v_id].add_activity(1.0);
                        self.variables[literal.v_id]
                            .note_occurence(&stored_clause, literal.polarity);
                    }

                    self.learnt_clauses.push(stored_clause.clone());
                    stored_clause
                }
            },
        }
    }

    pub fn drop_learnt_clause_by_swap(&mut self, index: usize) {
        let stored_clause = &self.learnt_clauses[index];

        let watched_a_lit = stored_clause.watched_a();
        self.variables[watched_a_lit.v_id].watch_removed(stored_clause, watched_a_lit.polarity);

        let watched_b_lit = stored_clause.watched_b();
        self.variables[watched_b_lit.v_id].watch_removed(stored_clause, watched_b_lit.polarity);

        for literal in stored_clause.literals() {
            self.variables[literal.v_id].note_drop(stored_clause, literal.polarity)
        }

        let _ = self.learnt_clauses.swap_remove(index);
    }

    pub fn unset_literal(&mut self, literal: Literal) {
        log::trace!("Unset: {}", literal);

        let v_id = literal.v_id;

        self.valuation[v_id] = None;
        self.variables[v_id].clear_decision_level();
    }
}

impl Solve<'_> {
    pub fn backjump(&mut self, to: LevelIndex) {
        log::trace!("Backjump from {} to {}", self.current_level().index(), to);

        for _ in 0..(self.current_level().index() - to) {
            let the_level = self.levels.pop().unwrap();
            for literal in the_level.literals() {
                self.unset_literal(literal);
            }
        }
    }
}
