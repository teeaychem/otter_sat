use crate::{
    context::stores::{variable::VariableStore, ClauseKey},
    structures::{
        clause::Clause,
        literal::{Literal, LiteralTrait},
        variable::list::VariableList,
    },
    types::clause::{WatchElement, WatchStatus},
};

use std::{borrow::Borrow, ops::Deref};

#[derive(Debug)]
pub struct StoredClause {
    key: ClauseKey,
    clause: Vec<Literal>,
    last: usize,
}

impl StoredClause {
    pub fn from(key: ClauseKey, clause: Vec<Literal>, variables: &mut VariableStore) -> Self {
        let mut stored_clause = Self {
            key,
            clause,
            last: 0,
        };

        stored_clause.initialise_watches(variables);

        stored_clause
    }

    pub const fn key(&self) -> ClauseKey {
        self.key
    }

    pub fn replace_key(&mut self, key: ClauseKey) {
        self.key = key
    }

    // pub fn original_clause(&self) -> Vec<Literal> {
    //     let mut original = self.clause.clone();
    //     // for hidden in &self.subsumed_literals {
    //     //     original.push(*hidden)
    //     // }
    //     original
    // }
}

// Watches

impl StoredClause {
    fn initialise_watches(&mut self, variables: &mut VariableStore) {
        let clause_length = self.clause.len() - 1;

        let mut index = 0;
        let watch_a = loop {
            if index == clause_length {
                break index;
            }

            let literal = self.clause[index];
            let literal_value = variables.value_of(literal);
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
            let index_value = variables.value_of(*index_literal);
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

    fn note_watch<L: Borrow<Literal>>(&self, literal: L, variables: &mut VariableStore) {
        let literal = literal.borrow();
        match self.key {
            ClauseKey::Binary(_) => {
                let check_literal = if self.clause[0].v_id() == literal.v_id() {
                    self.clause[1]
                } else {
                    self.clause[0]
                };

                variables.add_watch(literal, WatchElement::Binary(check_literal, self.key()));
            }
            ClauseKey::Formula(_) | ClauseKey::Learned(_, _) => {
                variables.add_watch(literal, WatchElement::Clause(self.key()));
            }
        }
    }

    #[inline(always)]
    #[allow(clippy::result_unit_err)]
    pub fn update_watch<L: Borrow<Literal>>(
        &mut self,
        literal: L,
        variables: &mut VariableStore,
    ) -> Result<WatchStatus, ()> {
        /*
        This will, logic issues aside, only be called on long formulas
        And, given how often it is called, checks to ensure there are no logic issues aren't worthwhile
        The assertion is commented for when needed
         */
        // assert!(self.clause.len() > 2);

        if unsafe { self.clause.get_unchecked(0).v_id() == literal.borrow().v_id() } {
            self.clause.swap(0, self.last)
        }
        /*
        The two for loops avoid the need to check whether the search pointer is equal to where the last search pointer stopped each time it's incremented
        Naive tests suggest there isn't a significant difference…
         */

        for i in (self.last + 1)..self.clause.len() {
            let last_literal = unsafe { self.clause.get_unchecked(i) };
            match variables.value_of(*last_literal) {
                None => {
                    self.last = i;
                    self.note_watch(*last_literal, variables);
                    return Ok(WatchStatus::None);
                }
                Some(value) if value == last_literal.polarity() => {
                    self.last = i;
                    self.note_watch(*last_literal, variables);
                    return Ok(WatchStatus::Witness);
                }
                Some(_) => {}
            }
        }

        for i in 1..self.last {
            let last_literal = unsafe { self.clause.get_unchecked(i) };
            match variables.value_of(*last_literal) {
                None => {
                    self.last = i;
                    self.note_watch(*last_literal, variables);
                    return Ok(WatchStatus::None);
                }
                Some(value) if value == last_literal.polarity() => {
                    self.last = i;
                    self.note_watch(*last_literal, variables);
                    return Ok(WatchStatus::Witness);
                }
                Some(_) => {}
            }
        }

        // let last_cache = self.last;
        // let clause_length = self.clause.len();
        // loop {
        //     self.last += 1;
        //     if self.last == clause_length {
        //         self.last = 1 // skip 0
        //     }
        //     if self.last == last_cache {
        //         return Err(());
        //     }
        //     let last_literal = unsafe { self.clause.get_unchecked(self.last) };
        //     match variables.value_of(last_literal.index()) {
        //         None => {
        //             self.note_watch(*last_literal, variables);
        //             return Ok(WatchStatus::None);
        //         }
        //         Some(value) if value == last_literal.polarity() => {
        //             self.note_watch(*last_literal, variables);
        //             return Ok(WatchStatus::Witness);
        //         }
        //         Some(_) => {}
        //     }
        // }
        Err(())
    }
}

// Subsumption

#[derive(Debug, Clone, Copy)]
pub enum SubsumptionError {
    ShortClause,
    NoPivot,
    WatchError,
}

impl StoredClause {
    /*
    Subsumption may result in the removal of a watched literal.
    If `fix_watch` is set then watches will be corrected after removing the literal.
    Watches may be left in a corrupted state as there may be no interest in fixing them.
    For example,  subsumption may lead to a binary clause and the watches for the clause may be set elsewhere.
    (This is what was implemented when this note was written…)

    For the moment subsumption does not allow subsumption to a unit clause

    TODO: FRAT adjustments
    At the moment learnt clauses are modified in place.
    For FRAT it's not clear whether id overwriting is ok.
     */
    pub fn subsume<L: Borrow<Literal>>(
        &mut self,
        literal: L,
        variables: &mut VariableStore,
        fix_watch: bool,
    ) -> Result<usize, SubsumptionError> {
        if self.clause.len() < 3 {
            log::error!(target: crate::log::targets::SUBSUMPTION, "Subsumption attempted on non-long clause");
            return Err(SubsumptionError::ShortClause);
        }
        let mut position = {
            let search = self
                .clause
                .iter()
                .position(|clause_literal| clause_literal == literal.borrow());
            match search {
                None => {
                    log::error!(target: crate::log::targets::SUBSUMPTION, "Pivot not found for subsumption");
                    return Err(SubsumptionError::NoPivot);
                }
                Some(p) => p,
            }
        };

        if position == 0 {
            self.clause.swap(0, self.last);
            position = self.last;
        }

        let removed = self.clause.swap_remove(position);
        // self.subsumed_literals.push(removed);

        match variables.remove_watch(removed, self.key) {
            Ok(()) => {}
            Err(_) => return Err(SubsumptionError::WatchError),
        };

        if fix_watch && position == self.last {
            let clause_length = self.clause.len();
            self.last = 1;
            for index in 1..clause_length {
                let index_literal = unsafe { self.clause.get_unchecked(index) };
                let index_value = variables.value_of(*index_literal);
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
        Ok(self.clause.len())
    }
}

impl std::fmt::Display for StoredClause {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.clause.as_string())
    }
}

impl Deref for StoredClause {
    type Target = [Literal];

    fn deref(&self) -> &Self::Target {
        &self.clause
    }
}
