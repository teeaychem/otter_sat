use slotmap::SlotMap;

use crate::structures::{
    clause::{
        stored_clause::{initialise_watches_for, ClauseSource, StoredClause, Watch},
        Clause,
    },
    formula::Formula,
    level::{Level, LevelIndex},
    literal::{Literal, LiteralSource},
    solve::the_solve::literal_update,
    solve::{
        clause_store::{retreive, ClauseKey, ClauseStore},
        Solve,
    },
    valuation::{Valuation, ValuationVec},
    variable::{Variable, VariableId},
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
            clauses_stored: ClauseStore {
                formula_clauses: SlotMap::new(),
                learnt_clauses: SlotMap::new(),
            },
        };

        let initial_valuation = the_solve.valuation.clone();

        for formula_clause in clauses {
            match formula_clause.length() {
                0 => {
                    panic!("c The formula contains a zero-length clause");
                }
                _ => {
                    let clause_key =
                        the_solve.store_clause(formula_clause.to_vec(), ClauseSource::Formula);

                    let stored_clause = retreive(&the_solve.clauses_stored, clause_key);

                    initialise_watches_for(stored_clause, &initial_valuation, &the_solve.variables);
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
        self.clauses_stored
            .formula_clauses
            .iter()
            .chain(&self.clauses_stored.learnt_clauses)
            .map(|(_, sc)| sc)
    }

    pub fn clauses(&self) -> impl Iterator<Item = &impl Clause> {
        self.stored_clauses()
            .map(|stored_clause| stored_clause.clause_impl())
    }

    pub fn var_by_id(&self, id: VariableId) -> Option<&Variable> {
        self.variables.get(id)
    }

    pub fn literal_set_from_vec(&mut self, choices: Vec<VariableId>) {
        choices.iter().for_each(|&v_id| {
            let the_literal = Literal::new(v_id, false);
            literal_update(
                the_literal,
                LiteralSource::HobsonChoice,
                &mut self.levels,
                &self.variables,
                &mut self.valuation,
                &self.clauses_stored,
            );
            self.watch_q.push_back(the_literal);
        });
    }

    pub fn most_active_none(&self, val: &impl Valuation) -> Option<VariableId> {
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
                    let key = self.clauses_stored.formula_clauses.insert_with_key(|k| {
                        StoredClause::new_from(ClauseKey::Formula(k), clause, src)
                    });

                    let bc = &self.clauses_stored.formula_clauses[key];

                    for literal in bc.literals() {
                        self.variables[literal.v_id]
                            .note_occurence(ClauseKey::Formula(key), literal.polarity);
                    }

                    ClauseKey::Formula(key)
                }
                ClauseSource::Resolution(_) => {
                    log::trace!("Learning clause {}", clause.as_string());
                    let key = self.clauses_stored.learnt_clauses.insert_with_key(|k| {
                        StoredClause::new_from(ClauseKey::Learnt(k), clause, src)
                    });

                    let bc = &self.clauses_stored.learnt_clauses[key];

                    for literal in bc.literals() {
                        self.variables[literal.v_id]
                            .note_occurence(ClauseKey::Learnt(key), literal.polarity);
                    }

                    ClauseKey::Learnt(key)
                }
            },
        }
    }

    pub fn drop_learnt_clause_by_swap(&mut self, clause_key: ClauseKey) {
        if let ClauseKey::Learnt(key) = clause_key {
            let stored_clause = &self.clauses_stored.learnt_clauses[key];

            let watched_a_lit = stored_clause.literal_of(Watch::A);
            self.variables[watched_a_lit.v_id].watch_removed(stored_clause, watched_a_lit.polarity);

            let watched_b_lit = stored_clause.literal_of(Watch::B);
            self.variables[watched_b_lit.v_id].watch_removed(stored_clause, watched_b_lit.polarity);

            for literal in stored_clause.literals() {
                self.variables[literal.v_id].note_clause_drop(clause_key, literal.polarity)
            }

            let _ = self.clauses_stored.learnt_clauses.remove(key);
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

                let v_id = literal.v_id;

                self.valuation[v_id] = None;
                self.variables[v_id].clear_decision_level();
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
