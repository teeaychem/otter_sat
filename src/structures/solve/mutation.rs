use crate::structures::{
    solve::Solve, stored_clause::suggest_watch_update, Clause, ClauseSource, LevelIndex, Literal,
    StoredClause, Valuation, Variable,
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
pub fn process_watches(
    valuation: &impl Valuation,
    variables: &mut [Variable],
    stored_clause: &Rc<StoredClause>,
    lit: Literal,
) -> bool {
    let (a_update, b_update, propagation_ready) =
        suggest_watch_update(stored_clause, valuation, lit.v_id, variables);


    match (a_update, b_update) {
        (Some(a), None) => {
            switch_watch_a(variables, stored_clause, a);
        }
        (None, Some(b)) => {
            switch_watch_b(variables, stored_clause, b);
        }
        (None, None) => (),
        _ => panic!("Unknown watch update"),
    };
    propagation_ready
}

#[inline(always)]
fn switch_watch_a(variables: &mut [Variable], stored_clause: &Rc<StoredClause>, index: usize) {
    let watched_a_lit = stored_clause.watched_a();
    variables[watched_a_lit.v_id].watch_removed(stored_clause, watched_a_lit.polarity);
    stored_clause.update_watch_a(index);
    variables[stored_clause.watched_a().v_id]
        .watch_added(stored_clause, stored_clause.watched_a().polarity)
}

#[inline(always)]
fn switch_watch_b(variables: &mut [Variable], stored_clause: &Rc<StoredClause>, index: usize) {
    let watched_b_lit = stored_clause.watched_b();
    variables[watched_b_lit.v_id].watch_removed(stored_clause, watched_b_lit.polarity);
    stored_clause.update_watch_b(index);
    variables[stored_clause.watched_b().v_id]
        .watch_added(stored_clause, stored_clause.watched_b().polarity)
}
