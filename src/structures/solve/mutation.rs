use crate::structures::{
    solve::{Solve, SolveError},
    stored_clause::suggest_watch_update,
    Clause, ClauseSource, LevelIndex, Literal, LiteralSource, StoredClause, Valuation,
    ValuationError,
};
use std::rc::Rc;

impl<'borrow, 'solve> Solve<'solve> {
    /// Stores a clause with an automatically generated id.
    /// Note: In order to use the clause the watch literals of the struct must be initialised.
    pub fn store_clause(
        &'borrow mut self,
        clause: impl Clause,
        src: ClauseSource,
    ) -> Rc<StoredClause> {
        match clause.len() {
            0 => panic!("Attempt to add an empty clause"),
            _ => match &src {
                ClauseSource::Formula => {
                    let stored_clause =
                        StoredClause::new_from(Solve::fresh_clause_id(), clause, src);

                    for literal in stored_clause.clause().literals() {
                        self.variables[literal.v_id]
                            .note_occurence(&stored_clause, literal.polarity);
                    }

                    self.formula_clauses.push(stored_clause.clone());
                    stored_clause
                }
                ClauseSource::Resolution(_) => {
                    log::warn!("Learning clause {}", clause.as_string());
                    let stored_clause =
                        StoredClause::new_from(Solve::fresh_clause_id(), clause, src);

                    for literal in stored_clause.clause().literals() {
                        self.variables[literal.v_id].increase_activity(1.0);
                        self.variables[literal.v_id]
                            .note_occurence(&stored_clause, literal.polarity);
                    }
                    self.learnt_clauses.push(stored_clause.clone());
                    stored_clause
                }
            },
        }
    }

    pub fn drop_clause(&mut self, stored_clause: &Rc<StoredClause>) {
        self.variables[stored_clause.watched_a().v_id].watch_removed(stored_clause);
        self.variables[stored_clause.watched_b().v_id].watch_removed(stored_clause);
        if let Some(p) = self
            .learnt_clauses
            .iter()
            .position(|sc| sc == stored_clause)
        {
            let _ = self.learnt_clauses.swap_remove(p);
        } else {
            panic!("Unable to remove: {} from learnt clauses", stored_clause);
        }
        for literal in stored_clause.clause().literals() {
            self.variables[literal.v_id].note_drop(literal.polarity, stored_clause)
        }
    }

    /*
    Primary function for setting a literal during a solve
    Control branches on the current valuation and then the source of the literal.
    A few things may be updated:
    - The current valuation.
    - Records at the current level.
    - The implication graph.
     */
    pub fn set_literal(
        &'borrow mut self,
        lit: Literal,
        src: LiteralSource,
    ) -> Result<(), SolveError> {
        log::trace!("Set literal: {} | src: {:?}", lit, src);
        match self.valuation.update_value(lit) {
            Ok(()) => {
                let level_index = match src {
                    LiteralSource::Choice => self.add_fresh_level(),
                    LiteralSource::Assumption | LiteralSource::HobsonChoice => 0,
                    LiteralSource::StoredClause(_) => self.current_level().index(),
                };
                {
                    let mut informative_literal = false;

                    for sc in 0..self.variables[lit.v_id].positive_occurrences().len() {
                        let stored_clause =
                            &self.variables[lit.v_id].positive_occurrences()[sc].clone();
                        self.process_watches(stored_clause, lit, &mut informative_literal);
                    }
                    for sc in 0..self.variables[lit.v_id].negative_occurrences().len() {
                        let stored_clause =
                            &self.variables[lit.v_id].negative_occurrences()[sc].clone();
                        self.process_watches(stored_clause, lit, &mut informative_literal);
                    }

                    if informative_literal {
                        self.current_level_mut().note_watch(lit)
                    }
                }
                match &src {
                    LiteralSource::Choice => {
                        self.current_level_mut().record_literal(lit, src);
                        self.variables[lit.v_id].set_decision_level(level_index);
                        log::debug!("+Set choice: {lit}");
                    }
                    LiteralSource::Assumption => {
                        self.variables[lit.v_id].set_decision_level(level_index);
                        self.top_level_mut().record_literal(lit, src);
                        log::debug!("+Set assumption: {lit}");
                    }
                    LiteralSource::HobsonChoice => {
                        self.variables[lit.v_id].set_decision_level(level_index);
                        self.top_level_mut().record_literal(lit, src);
                        log::debug!("+Set hobson choice: {lit}");
                    }
                    LiteralSource::StoredClause(_) => {
                        self.variables[lit.v_id].set_decision_level(level_index);
                        self.current_level_mut().record_literal(lit, src);
                        log::debug!("+Set deduction: {lit}");
                    }
                };
                Ok(())
            }
            Err(ValuationError::Match) => match src {
                LiteralSource::StoredClause(_) => {
                    // A literal may be implied by multiple clauses
                    Ok(())
                }
                _ => {
                    log::error!("Attempting to restate {} via {:?}", lit, src);
                    panic!("Attempting to restate the valuation")
                }
            },
            Err(ValuationError::Conflict) => {
                match src {
                    LiteralSource::StoredClause(id) => {
                        // A literal may be implied by multiple clauses
                        Err(SolveError::Conflict(id, lit))
                    }
                    _ => {
                        log::error!("Attempting to flip {} via {:?}", lit, src);
                        panic!("Attempting to flip the valuation")
                    }
                }
            }
        }
    }

    pub fn unset_literal(&mut self, literal: Literal) {
        log::trace!("Unset: {}", literal);

        let v_id = literal.v_id;

        self.valuation[v_id] = None;
        self.variables[v_id].clear_decision_level();
    }

    fn process_watches(
        &mut self,
        stored_clause: &Rc<StoredClause>,
        lit: Literal,
        informative_literal: &mut bool,
    ) {
        match suggest_watch_update(stored_clause, &self.valuation, lit.v_id, self.variables()) {
            (Some(a), None, true) => {
                self.switch_watch_a(stored_clause, a);
                *informative_literal = true
            }
            (None, Some(b), true) => {
                self.switch_watch_b(stored_clause, b);

                *informative_literal = true
            }
            (Some(a), None, false) => {
                self.switch_watch_a(stored_clause, a);
            }
            (None, Some(b), false) => {
                self.switch_watch_b(stored_clause, b);
            }
            (None, None, true) => *informative_literal = true,
            (None, None, false) => (),
            _ => panic!("Unknown watch update"),
        };
    }
}

impl Solve<'_> {
    pub fn backjump(&mut self, to: LevelIndex) {
        log::warn!("Backjump from {} to {}", self.current_level().index(), to);

        for _ in 0..(self.current_level().index() - to) {
            let the_level = self.levels.pop().unwrap();
            for literal in the_level.literals() {
                self.unset_literal(literal);
            }
        }
    }
}
