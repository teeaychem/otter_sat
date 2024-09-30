use crate::structures::{
    solve::{Solve, SolveError},
    stored_clause::suggest_watch_update,
    Clause, ClauseSource, Level, LevelIndex, Literal, LiteralSource, StoredClause, Valuation,
    ValuationError, Variable,
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
                    log::warn!("Learning clause {}", clause.as_string());
                    let stored_clause =
                        StoredClause::new_from(Solve::fresh_clause_id(), clause, src);

                    for literal in stored_clause.literals() {
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

    pub fn drop_clause_by_swap(&mut self, stored_clause: &Rc<StoredClause>) {
        let watched_a_lit = stored_clause.watched_a();
        let watched_b_lit = stored_clause.watched_b();
        self.variables[watched_a_lit.v_id].watch_removed(stored_clause, watched_a_lit.polarity);
        self.variables[watched_b_lit.v_id].watch_removed(stored_clause, watched_b_lit.polarity);
        if let Some(p) = self
            .learnt_clauses
            .iter()
            .position(|sc| sc == stored_clause)
        {
            let _ = self.learnt_clauses.swap_remove(p);
        } else {
            panic!("Unable to remove: {} from learnt clauses", stored_clause);
        }
        for literal in stored_clause.literals() {
            self.variables[literal.v_id].note_drop(literal.polarity, stored_clause)
        }
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
        log::warn!("Backjump from {} to {}", self.current_level().index(), to);

        for _ in 0..(self.current_level().index() - to) {
            let the_level = self.levels.pop().unwrap();
            for literal in the_level.literals() {
                self.unset_literal(literal);
            }
        }
    }
}

#[inline(always)]
pub fn process_variable_occurrence_update(
    valuation: &impl Valuation,
    variables: &mut [Variable],
    lit: Literal,
) -> bool {
    let mut informative_literal = false;

    for sc in 0..variables[lit.v_id].positive_occurrences().len() {
        let stored_clause = variables[lit.v_id].positive_occurrences()[sc].clone();
        process_watches(
            valuation,
            variables,
            &stored_clause,
            lit,
            &mut informative_literal,
        );
    }
    for sc in 0..variables[lit.v_id].negative_occurrences().len() {
        let stored_clause = variables[lit.v_id].negative_occurrences()[sc].clone();
        process_watches(
            valuation,
            variables,
            &stored_clause,
            lit,
            &mut informative_literal,
        );
    }

    informative_literal
}

#[inline(always)]
fn process_watches(
    valuation: &impl Valuation,
    variables: &mut [Variable],
    stored_clause: &Rc<StoredClause>,
    lit: Literal,
    informative_literal: &mut bool,
) {
    match suggest_watch_update(stored_clause, valuation, lit.v_id, variables) {
        (Some(a), None, true) => {
            switch_watch_a(variables, stored_clause, a);
            *informative_literal = true
        }
        (None, Some(b), true) => {
            switch_watch_b(variables, stored_clause, b);

            *informative_literal = true
        }
        (Some(a), None, false) => {
            switch_watch_a(variables, stored_clause, a);
        }
        (None, Some(b), false) => {
            switch_watch_b(variables, stored_clause, b);
        }
        (None, None, true) => *informative_literal = true,
        (None, None, false) => (),
        _ => panic!("Unknown watch update"),
    };
}

#[inline(always)]
fn switch_watch_a(variables: &mut [Variable], stored_clause: &Rc<StoredClause>, index: usize) {
    let watched_a_lit = stored_clause.watched_a();
    variables[watched_a_lit.v_id].watch_removed(stored_clause, watched_a_lit.polarity);
    stored_clause.update_watch_a(index);
    variables[stored_clause.watched_a().v_id].watch_added(stored_clause, stored_clause.watched_a().polarity)
}

#[inline(always)]
fn switch_watch_b(variables: &mut [Variable], stored_clause: &Rc<StoredClause>, index: usize) {
    let watched_b_lit = stored_clause.watched_b();
    variables[watched_b_lit.v_id].watch_removed(stored_clause, watched_b_lit.polarity);
    stored_clause.update_watch_b(index);
    variables[stored_clause.watched_b().v_id].watch_added(stored_clause, stored_clause.watched_b().polarity)
}

/*
Primary function for figuring out the consequences of setting a literal during a solve.
Control branches on the current valuation and then the source of the literal.
A few things may be updated:
- The current valuation.
- Records at the current level.
 */
#[inline(always)]
pub fn process_update_literal(
    lit: Literal,
    src: LiteralSource,
    variable: &mut Variable,
    levels: &mut [Level],
    literal_update_result: Result<(), ValuationError>,
) -> Result<(), SolveError> {
    match literal_update_result {
        Ok(()) => {
            match &src {
                LiteralSource::Choice => {
                    let current_level = levels.len() - 1;
                    variable.set_decision_level(current_level);
                    levels[current_level].record_literal(lit, src);
                    log::debug!("+Set choice: {lit}");
                }
                LiteralSource::StoredClause(_) => {
                    let current_level = levels.len() - 1;
                    variable.set_decision_level(current_level);
                    levels[current_level].record_literal(lit, src);
                    log::debug!("+Set deduction: {lit}");
                }
                LiteralSource::Assumption | LiteralSource::HobsonChoice => {
                    variable.set_decision_level(0);
                    levels[0].record_literal(lit, src);
                    log::debug!("+Set assumption/hobson choice: {lit}");
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
        Err(ValuationError::Conflict) => match src {
            LiteralSource::StoredClause(id) => Err(SolveError::Conflict(id, lit)),
            _ => {
                log::error!("Attempting to flip {} via {:?}", lit, src);
                panic!("Attempting to flip the valuation")
            }
        },
    }
}
