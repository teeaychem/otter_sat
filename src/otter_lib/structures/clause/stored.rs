use crate::{
    context::store::ClauseKey,
    structures::{
        clause::Clause,
        literal::Literal,
        variable::{list::VariableList, Variable},
    },
};

use std::ops::Deref;

#[derive(Debug)]
pub struct StoredClause {
    key: ClauseKey,
    source: Source,
    clause: Vec<Literal>,
    subsumed_literals: Vec<Literal>,
}

// { Clause enums

#[derive(Clone, Copy, Debug)]
pub enum Source {
    Formula,
    Resolution,
}

// }

// { Watch enums

#[derive(Clone, Copy, Debug)]
pub enum Watch {
    A,
    B,
}

#[derive(Clone, Copy, PartialEq)]
pub enum WatchStatus {
    Witness,
    None,
    Conflict,
}

// }

impl StoredClause {
    pub fn new_from(
        key: ClauseKey,
        clause: Vec<Literal>,
        source: Source,
        variables: &impl VariableList,
    ) -> Self {
        let figured_out = figure_out_intial_watches(clause, variables);
        let stored_clause = Self {
            key,
            source,
            clause: figured_out,
            subsumed_literals: vec![],
        };

        let watched_a = stored_clause.get_watch(Watch::A);
        let watched_b = stored_clause.get_watch(Watch::B);

        variables
            .get_unsafe(watched_a.index())
            .watch_added(stored_clause.key, watched_a.polarity());

        variables
            .get_unsafe(watched_b.index())
            .watch_added(stored_clause.key, watched_b.polarity());

        stored_clause
    }

    pub const fn key(&self) -> ClauseKey {
        self.key
    }

    pub const fn source(&self) -> Source {
        self.source
    }

    pub fn get_watch(&self, watch: Watch) -> Literal {
        match watch {
            Watch::A => unsafe { *self.clause.get_unchecked(0) },
            Watch::B => unsafe { *self.clause.get_unchecked(1) },
        }
    }

    #[inline(always)]
    fn watch_update_replace(
        &mut self,
        watch: Watch,
        index: usize,
        variable: &Variable,
        literal: Literal,
    ) {
        match watch {
            Watch::A => {
                self.clause.swap(index, 0);
                variable.watch_added(self.key, literal.polarity());
            }
            Watch::B => {
                self.clause.swap(index, 1);
                variable.watch_added(self.key, literal.polarity());
            }
        };
    }

    #[inline(always)]
    #[allow(clippy::result_unit_err)]
    /// Searches for and then updates to a new literal for the given watch index
    /// Returns true if the the watch was updated
    /// The match is to help prototype re-ordering the clause
    /// Specifically, the general case allows storing information about the previous literal
    pub fn update_watch(
        &mut self,
        watch: Watch,
        variables: &impl VariableList,
    ) -> Result<WatchStatus, ()> {
        match self.clause.len() {
            2 => {
                match variables.polarity_of(self.get_watch(watch).index()) {
                    None => return Ok(WatchStatus::None),
                    Some(polarity) if polarity == self.get_watch(watch).polarity() => {
                        return Ok(WatchStatus::Witness)
                    }
                    Some(_) => return Err(()),
                };
            }
            3 => {
                let the_literal = unsafe { *self.clause.get_unchecked(2) };
                let the_variable = variables.get_unsafe(the_literal.index());
                match the_variable.value() {
                    None => {
                        self.watch_update_replace(watch, 2, the_variable, the_literal);
                        return Ok(WatchStatus::None);
                    }
                    Some(polarity) if polarity == the_literal.polarity() => {
                        self.watch_update_replace(watch, 2, the_variable, the_literal);
                        return Ok(WatchStatus::Witness);
                    }
                    Some(_) => {}
                }
            }
            _ => {
                for index in 2..self.clause.len() {
                    let the_literal = unsafe { *self.clause.get_unchecked(index) };
                    let the_variable = variables.get_unsafe(the_literal.index());

                    match the_variable.value() {
                        None => {
                            self.watch_update_replace(watch, index, the_variable, the_literal);
                            return Ok(WatchStatus::None);
                        }
                        Some(polarity) if polarity == the_literal.polarity() => {
                            self.watch_update_replace(watch, index, the_variable, the_literal);
                            return Ok(WatchStatus::Witness);
                        }
                        Some(_) => {}
                    }
                }
            }
        }
        Err(())
    }

    /// 'Subsumes' a clause by removing the given literal.
    /// Records the clause has been subsumed, but does not store a record.
    /// In order to keep a record of the clauses used to prove the subsumption, use `literal_subsumption_core`.
    /// Returns Ok(()) if subsumption was ok, Err(()) otherwise
    #[allow(clippy::result_unit_err)]
    pub fn subsume(&mut self, literal: Literal, variables: &impl VariableList) -> Result<(), ()> {
        if self.clause.len() > 2 {
            if let Some(position) = self
                .clause
                .iter()
                .position(|clause_literal| *clause_literal == literal)
            {
                let removed = self.clause.swap_remove(position);
                if removed == unsafe { *self.clause.get_unchecked(0) } {
                    let _ = self.update_watch(Watch::A, variables);
                } else if removed == unsafe { *self.clause.get_unchecked(1) } {
                    let _ = self.update_watch(Watch::B, variables);
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
}

impl std::fmt::Display for StoredClause {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.clause.as_string())
    }
}

fn figure_out_intial_watches(mut clause: Vec<Literal>, val: &impl VariableList) -> Vec<Literal> {
    let mut watch_a = 0;
    let mut watch_b = 1;
    let mut a_status = get_status(unsafe { *clause.get_unchecked(watch_a) }, val);
    let mut b_status = get_status(unsafe { *clause.get_unchecked(watch_b) }, val);

    /*
    The initial setup gurantees a has status none or witness, while b may have any status
    priority is given to watch a, so that watch b remains a conflict until watch a becomes none
    at which point, b inherits the witness status of a (which may be updated again) or becomes none and no more checks need to happen
     */

    for index in 2..clause.len() {
        if a_status == WatchStatus::None && b_status == WatchStatus::None {
            break;
        }
        let literal = unsafe { *clause.get_unchecked(index) };
        let literal_status = get_status(literal, val);
        match literal_status {
            WatchStatus::Conflict => {} // do nothing on a conflict
            WatchStatus::None => {
                // by the first check, either a or b fails to be none, so update a or otherwise b
                if a_status != WatchStatus::None {
                    // though, if a is acting as a witness, pass this to b
                    if a_status == WatchStatus::Witness {
                        watch_b = watch_a;
                        watch_a = index;
                        a_status = WatchStatus::None;
                        b_status = WatchStatus::Witness;
                    } else {
                        watch_a = index;
                        a_status = WatchStatus::None;
                    }
                } else {
                    watch_b = index;
                    b_status = WatchStatus::None;
                }
            }
            WatchStatus::Witness => {
                if a_status == WatchStatus::Conflict {
                    watch_a = index;
                    a_status = WatchStatus::Witness;
                }
            }
        }

        clause.swap(0, watch_a);
        clause.swap(1, watch_b);
    }

    clause
}

fn get_status(literal: Literal, variables: &impl VariableList) -> WatchStatus {
    match variables.polarity_of(literal.index()) {
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
