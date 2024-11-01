use crate::{
    config::ClauseActivity,
    context::store::ClauseKey,
    structures::{
        clause::Clause,
        literal::Literal,
        variable::{delegate::VariableStore, list::VariableList, WatchElement},
    },
};

use std::ops::Deref;

#[derive(Debug)]
pub struct StoredClause {
    key: ClauseKey,
    source: ClauseSource,
    clause: Vec<Literal>,
    subsumed_literals: Vec<Literal>,
    pub last: usize,
    pub activity: ClauseActivity,
}

#[derive(Clone, Copy, Debug)]
pub enum ClauseSource {
    Formula,
    Resolution,
}

#[derive(Clone, Copy, PartialEq)]
pub enum WatchStatus {
    Witness,
    None,
    Conflict,
    TwoWitness,
    TwoNone,
    TwoConflict,
}

impl StoredClause {
    pub fn new_from(
        key: ClauseKey,
        clause: Vec<Literal>,
        subsumed: Vec<Literal>,
        source: ClauseSource,
        variables: &VariableStore,
    ) -> Self {
        let mut stored_clause = Self {
            key,
            source,
            clause,
            subsumed_literals: subsumed,
            last: 0,
            activity: 1.0,
        };

        stored_clause.initialise_watches(variables);

        stored_clause
    }

    pub const fn key(&self) -> ClauseKey {
        self.key
    }

    pub const fn source(&self) -> ClauseSource {
        self.source
    }

    #[inline(always)]
    #[allow(clippy::result_unit_err)]
    /// Searches for and then updates to a new literal for the given watch index
    /// Returns true if the the watch was updated
    /// The match is to help prototype re-ordering the clause
    /// Specifically, the general case allows storing information about the previous literal
    pub fn update_watch(
        &mut self,
        literal: Literal,
        variables: &impl VariableList,
    ) -> Result<WatchStatus, ()> {
        match self.clause.len() {
            2 => {
                if unsafe { self.clause.get_unchecked(0).v_id() == literal.v_id() } {
                    self.clause.swap(0, 1)
                }
                let other_literal = unsafe { self.clause.get_unchecked(1) };
                match variables.value_of(other_literal.index()) {
                    None => Ok(WatchStatus::TwoNone),
                    Some(polarity) if polarity == other_literal.polarity() => {
                        Ok(WatchStatus::TwoWitness)
                    }
                    Some(_) => Err(()),
                }
            }
            _ => {
                if unsafe { self.clause.get_unchecked(0).v_id() == literal.v_id() } {
                    self.clause.swap(0, self.last)
                }
                let last_cache = self.last;
                let clause_length = self.clause.len();
                loop {
                    self.last += 1;
                    if self.last == clause_length {
                        self.last = 1 // skip 0
                    }
                    if self.last == last_cache {
                        return Err(());
                    }
                    let last_literal = unsafe { self.clause.get_unchecked(self.last) };
                    let last_value = variables.value_of(last_literal.index());
                    match last_value {
                        None => {
                            self.note_watch(*last_literal, variables);
                            return Ok(WatchStatus::None);
                        }
                        Some(value) if value == last_literal.polarity() => {
                            self.note_watch(*last_literal, variables);
                            return Ok(WatchStatus::Witness);
                        }
                        Some(_) => {}
                    }
                }
            }
        }
    }

    /// 'Subsumes' a clause by removing the given literal.
    /// Records the clause has been subsumed, but does not store a record.
    /// In order to keep a record of the clauses used to prove the subsumption, use `literal_subsumption_core`.
    /// Returns Ok(()) if subsumption was ok, Err(()) otherwise
    #[allow(clippy::result_unit_err)]
    pub fn subsume(&mut self, literal: Literal, variables: &impl VariableList) -> Result<(), ()> {
        if self.clause.len() > 2 {
            if let Some(mut position) = self
                .clause
                .iter()
                .position(|clause_literal| *clause_literal == literal)
            {
                if position == 0 {
                    self.clause.swap(0, self.last);
                    position = self.last;
                }
                let removed = self.clause.swap_remove(position);
                variables
                    .get_unsafe(removed.index())
                    .watch_removed(self.key, removed.polarity());
                if position == self.last {
                    let clause_length = self.clause.len();
                    self.last = 1;
                    for index in 1..clause_length {
                        let index_literal = unsafe { self.clause.get_unchecked(index) };
                        let index_value = variables.value_of(index_literal.index());
                        match index_value {
                            None => {
                                self.last = index;
                                break;
                            }
                            Some(value) if value == index_literal.polarity() => {
                                self.last = index;
                                break;
                            }
                            Some(_) => {}
                        }
                    }
                    self.note_watch(self.clause[self.last], variables);
                }

                self.subsumed_literals.push(removed);
                Ok(())
            } else {
                Err(())
            }
        } else {
            Err(())
        }
    }

    pub fn original_clause(&self) -> Vec<Literal> {
        let mut original = self.clause.clone();
        for hidden in &self.subsumed_literals {
            original.push(*hidden)
        }
        original
    }

    fn initialise_watches(&mut self, variables: &impl VariableList) {
        let clause_length = self.clause.len();

        let mut index = 0;
        let watch_a = loop {
            if index == clause_length {
                panic!("could not initialise watches for clause");
            }

            let literal = self.clause[index];
            let literal_value = variables.value_of(literal.index());
            match literal_value {
                None => break index,
                Some(value) if value == literal.polarity() => break index,
                Some(_) => index += 1,
            }
        };

        self.clause.swap(0, watch_a);

        self.last = 1;
        for index in 1..clause_length {
            let index_literal = unsafe { self.clause.get_unchecked(index) };
            let index_value = variables.value_of(index_literal.index());
            match index_value {
                None => {
                    self.last = index;
                    break;
                }
                Some(value) if value == index_literal.polarity() => {
                    self.last = index;
                    break;
                }
                Some(_) => {}
            }
        }

        self.note_watch(self.clause[0], variables);
        self.note_watch(self.clause[self.last], variables);
    }

    fn note_watch(&self, literal: Literal, variables: &impl VariableList) {
        match self.clause.len() {
            2 => {
                let check_literal = if self.clause[0].v_id() == literal.v_id() {
                    self.clause[1]
                } else {
                    self.clause[0]
                };

                variables.get_unsafe(literal.index()).watch_added(
                    WatchElement::Binary(check_literal, self.key()),
                    literal.polarity(),
                );
            }
            _ => {
                variables
                    .get_unsafe(literal.index())
                    .watch_added(WatchElement::Clause(self.key()), literal.polarity());
            }
        }
    }
}

impl std::fmt::Display for StoredClause {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.clause.as_string())
    }
}

fn get_status(literal: Literal, variables: &impl VariableList) -> WatchStatus {
    match variables.value_of(literal.index()) {
        None => WatchStatus::None,
        Some(polarity) if polarity == literal.polarity() => WatchStatus::Witness,
        Some(_) => WatchStatus::Conflict,
    }
}

impl Deref for StoredClause {
    type Target = [Literal];

    fn deref(&self) -> &Self::Target {
        &self.clause
    }
}
