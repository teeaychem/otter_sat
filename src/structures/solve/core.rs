use slotmap::SlotMap;

use crate::structures::{
    clause::{
        stored_clause::{ClauseSource, StoredClause, Watch},
        Clause,
    },
    formula::Formula,
    level::{Level, LevelIndex},
    solve::{ClauseKey, Solve},
    valuation::{Valuation, ValuationVec},
};

use std::collections::VecDeque;

impl Solve {
    pub fn from_formula(formula: Formula) -> Solve {
        let variables = formula.variables;
        let clauses = formula.clauses;

        let mut the_solve = Solve {
            conflicts: 0,
            conflicts_since_last_forget: 0,
            conflicts_since_last_reset: 0,
            restarts: 0,
            watch_q: VecDeque::with_capacity(variables.len() / 2),
            valuation: Vec::<Option<bool>>::new_for_variables(variables.len()),
            variables,
            levels: vec![Level::new(0)],
            formula_clauses: SlotMap::new(),
            learnt_clauses: SlotMap::new(),
        };

        for formula_clause in clauses {
            match formula_clause.length() {
                n if n < 2 => {
                    panic!("c The formula contains a zero-or-one-length clause");
                }
                _ => {
                    the_solve.store_clause(formula_clause.to_vec(), ClauseSource::Formula);
                }
            }
        }

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

    pub fn stored_clauses(&self) -> impl Iterator<Item = &StoredClause> {
        self.formula_clauses
            .iter()
            .chain(&self.learnt_clauses)
            .map(|(_, sc)| sc)
    }

    pub fn most_active_none(&self, val: &impl Valuation) -> Option<usize> {
        val.values()
            .enumerate()
            .filter(|(_, v)| v.is_none())
            .map(|(i, _)| (i, self.variables[i].activity()))
            .max_by(|a, b| a.1.total_cmp(&b.1))
            .map(|(a, _)| a)
    }

    /// Stores a clause with an automatically generated id.
    /// Note: In order to use the clause the watch literals of the struct must be initialised.
    pub fn store_clause(&mut self, clause: impl Clause, src: ClauseSource) -> ClauseKey {
        match clause.length() {
            0 => panic!("Attempt to add an empty clause"),
            _ => match &src {
                ClauseSource::Formula => {
                    let key = self.formula_clauses.insert_with_key(|k| {
                        StoredClause::new_from(
                            ClauseKey::Formula(k),
                            clause.to_vec(),
                            src,
                            &self.valuation,
                            &mut self.variables,
                        )
                    });

                    ClauseKey::Formula(key)
                }
                ClauseSource::Resolution(_) => {
                    log::trace!("Learning clause {}", clause.as_string());

                    let key = self.learnt_clauses.insert_with_key(|k| {
                        StoredClause::new_from(
                            ClauseKey::Learnt(k),
                            clause.to_vec(),
                            src,
                            &self.valuation,
                            &mut self.variables,
                        )
                    });

                    ClauseKey::Learnt(key)
                }
            },
        }
    }

    pub fn drop_learnt_clause(&mut self, clause_key: ClauseKey) {
        if let ClauseKey::Learnt(key) = clause_key {
            let stored_clause = &self.learnt_clauses[key];

            unsafe {
                let watched_a_lit = stored_clause.get_watched(Watch::A);
                self.variables
                    .get_unchecked(watched_a_lit.v_id())
                    .watch_removed(stored_clause.key(), watched_a_lit.polarity);

                let watched_b_lit = stored_clause.get_watched(Watch::B);
                self.variables
                    .get_unchecked(watched_b_lit.v_id())
                    .watch_removed(stored_clause.key(), watched_b_lit.polarity);
            }

            self.learnt_clauses.remove(key);
        } else {
            panic!("hek")
        }
    }

    pub fn backjump(&mut self, to: LevelIndex) {
        log::trace!("Backjump from {} to {}", self.current_level().index(), to);

        for _ in 0..(self.current_level().index() - to) {
            let the_level = self.levels.pop().unwrap();
            for literal in the_level.literals() {
                log::trace!("Unset: {}", literal);

                unsafe {
                    let v_id = literal.v_id();
                    *self.valuation.get_unchecked_mut(v_id) = None;
                    self.variables.get_unchecked(v_id).clear_decision_level();
                }
            }
        }
    }
}

impl std::fmt::Display for Solve {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let _ = writeln!(f, "Valuation: {}", self.valuation.as_display_string(self));
        let _ = write!(f, "More to be added…");
        Ok(())
    }
}
