use slotmap::SlotMap;

use crate::structures::{
    clause::{
        stored::{Source, StoredClause},
        vec::ClauseVec,
        Clause,
    },
    formula::Formula,
    level::{Level, LevelIndex},
    solve::{ClauseKey, Solve},
    valuation::{Valuation, ValuationVec},
};

use std::{collections::VecDeque, time::Duration};

impl Solve {
    pub fn from_formula(formula: Formula) -> Self {
        let variables = formula.variables;
        let clauses = formula.clauses;

        let mut the_solve = Self {
            time: Duration::new(0, 0),
            iterations: 0,
            conflicts: 0,
            conflicts_since_last_forget: 0,
            conflicts_since_last_reset: 0,
            restarts: 0,
            consequence_q: VecDeque::with_capacity(variables.len()),
            valuation: vec![None; variables.len()].into_boxed_slice(),
            variables,
            levels: Vec::<Level>::with_capacity(4096),
            formula_clauses: SlotMap::new(),
            learnt_clauses: SlotMap::new(),
        };
        the_solve.levels.push(Level::new(0));

        for formula_clause in clauses {
            assert!(
                formula_clause.len() > 1,
                "c The formula contains a zero-or-one-length clause"
            );

            the_solve.store_clause(formula_clause.to_clause_vec(), Source::Formula);
        }

        the_solve
    }

    pub fn valuation_at(&self, level_index: LevelIndex) -> ValuationVec {
        let mut valuation = vec![None; self.valuation.len()];
        (0..=level_index).for_each(|i| {
            self.levels[i].literals().for_each(|l| {
                valuation.set_value(l);
            });
        });
        valuation
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
    /// In order to use the clause the watch literals of the struct must be initialised.
    pub fn store_clause(&mut self, clause: ClauseVec, src: Source) -> ClauseKey {
        assert!(!clause.is_empty(), "Attempt to add an empty clause");

        match &src {
            Source::Formula => {
                let key = self.formula_clauses.insert_with_key(|k| {
                    StoredClause::new_from(
                        ClauseKey::Formula(k),
                        clause.to_clause_vec(),
                        src,
                        &self.valuation,
                        &mut self.variables,
                    )
                });

                ClauseKey::Formula(key)
            }
            Source::Resolution(_) => {
                log::trace!("Learning clause {}", clause.as_string());

                let key = self.learnt_clauses.insert_with_key(|k| {
                    let clause = StoredClause::new_from(
                        ClauseKey::Learnt(k),
                        clause.to_clause_vec(),
                        src,
                        &self.valuation,
                        &mut self.variables,
                    );
                    clause.set_lbd(&self.variables);
                    clause
                });

                ClauseKey::Learnt(key)
            }
        }
    }

    pub fn backjump(&mut self, to: LevelIndex) {
        log::trace!("Backjump from {} to {}", self.level().index(), to);

        for _ in 0..(self.level().index() - to) {
            for literal in self.levels.pop().expect("Lost level").literals() {
                log::trace!("Noneset: {}", literal.index());

                unsafe {
                    *self.valuation.get_unchecked_mut(literal.index()) = None;
                    self.variables
                        .get_unchecked(literal.index())
                        .clear_decision_level();
                }
            }
        }
    }
}
