use crate::{
    context::store::{ClauseKey, ClauseStore},
    structures::{
        clause::stored::Watch,
        level::Level,
        literal::{Literal, Source},
        variable::{list::VariableList, Variable},
    },
};

pub enum Status {
    NotSet,
    Match,
    Conflict,
}

use std::{
    collections::VecDeque,
    ops::{Deref, DerefMut},
};

pub struct VariableStore {
    variables: Vec<Variable>,
    consequence_q: VecDeque<Literal>,
    pub consequence_buffer: Vec<(Source, Literal)>,
}

impl VariableStore {
    pub fn new(variables: Vec<Variable>) -> Self {
        let count = variables.len();

        VariableStore {
            variables,
            consequence_q: VecDeque::with_capacity(count),
            consequence_buffer: Vec::with_capacity(count),
        }
    }
}

impl VariableStore {
    pub fn examine_consequences(
        &mut self,
        literal: Literal,
        level: &mut Level,
        stored_clauses: &ClauseStore,
    ) -> Result<(), ClauseKey> {
        let the_variable = self.variables.get_unsafe(literal.index());
        let occurrence_polarity = !literal.polarity();

        let mut index = 0;
        let mut length = the_variable.occurrence_length(occurrence_polarity);

        while index < length {
            let clause_key = the_variable.occurrence_key_at_index(occurrence_polarity, index);

            let stored_clause = stored_clauses.retreive(clause_key);

            let watch_a = stored_clause.get_watch(Watch::A);
            let watch_b = stored_clause.get_watch(Watch::B);

            the_variable.polarity();

            if watch_a.v_id() != literal.v_id() && watch_b.v_id() != literal.v_id() {
                the_variable.remove_occurrence_at_index(occurrence_polarity, index);
                length -= 1;
            } else {
                // the compiler prefers the conditional matches
                index += 1;
                let a_value = self.polarity_of(watch_a.index());
                let b_value = self.polarity_of(watch_b.index());

                match (a_value, b_value) {
                    (None, None) => {}
                    (Some(a), None) if a == watch_a.polarity() => {}
                    (Some(_), None) => {
                        self.set_value(watch_b, level.index());
                        // self.consequence_buffer
                        //     .push((Source::Clause(stored_clause.node_index()), watch_b));
                        level.record_literal(watch_b, Source::Clause(stored_clause.node_index()));
                        self.consequence_q.push_back(watch_b);
                    }
                    (None, Some(b)) if b == watch_b.polarity() => {}
                    (None, Some(_)) => {
                        self.set_value(watch_a, level.index());
                        // self.consequence_buffer
                        //     .push((Source::Clause(stored_clause.node_index()), watch_a));
                        level.record_literal(watch_a, Source::Clause(stored_clause.node_index()));
                        self.consequence_q.push_back(watch_a);
                    }
                    (Some(a), Some(b)) if a == watch_a.polarity() || b == watch_b.polarity() => {}
                    (Some(_), Some(_)) => {
                        // clean the watch lists while clearing the q
                        self.clear_queued_consequences(stored_clauses);
                        return Err(clause_key);
                    }
                }
            }
        }
        Ok(())
    }

    // lazy removals as implemented allow the lists to get quite messy if not kept clean
    fn clear_queued_consequences(&mut self, stored_clauses: &ClauseStore) {
        while let Some(literal) = self.consequence_q.pop_front() {
            let occurrences = {
                let the_variable = self.variables.get_unsafe(literal.index());
                match literal.polarity() {
                    true => unsafe { &mut *the_variable.negative_occurrences.get() },
                    false => unsafe { &mut *the_variable.positive_occurrences.get() },
                }
            };

            let mut index = 0;
            let mut length = occurrences.len();

            while index < length {
                let clause_key = unsafe { *occurrences.get_unchecked(index) };

                match stored_clauses.retreive_carefully(clause_key) {
                    Some(stored_clause) => {
                        let watch_a = stored_clause.get_watch(Watch::A);
                        let watch_b = stored_clause.get_watch(Watch::B);

                        if watch_a.v_id() != literal.v_id() && watch_b.v_id() != literal.v_id() {
                            occurrences.swap_remove(index);
                            length -= 1;
                        } else {
                            index += 1;
                        }
                    }
                    None => {
                        occurrences.swap_remove(index);
                        length -= 1;
                    }
                }
            }
        }
    }

    pub fn pop_front_consequence(&mut self) -> Option<Literal> {
        self.consequence_q.pop_front()
    }

    pub fn push_back_consequence(&mut self, literal: Literal) {
        self.consequence_q.push_back(literal)
    }
}

impl Deref for VariableStore {
    type Target = [Variable];

    fn deref(&self) -> &Self::Target {
        &self.variables
    }
}

impl DerefMut for VariableStore {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.variables
    }
}
