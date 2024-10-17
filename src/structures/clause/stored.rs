use crate::{
    context::store::{ClauseId, ClauseKey},
    structures::{
        clause::Clause,
        literal::Literal,
        variable::{variable_store::VariableStore, Variable},
    },
};

use petgraph::graph::NodeIndex;
use std::cell::UnsafeCell;
use std::ops::Deref;

pub struct StoredClause {
    id: ClauseId,
    key: ClauseKey,
    lbd: UnsafeCell<usize>,
    source: Source,
    clause: Vec<Literal>,
    cached_a: Literal,
    cached_b: Literal,
    subsumed: Vec<Literal>,
    node_index: Option<NodeIndex>,
}

// { Clause enums

#[derive(Clone, Debug)]
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
enum WatchStatus {
    Witness,
    None,
    Conflict,
}

// }

impl StoredClause {
    pub fn new_from(
        id: ClauseId,
        key: ClauseKey,
        clause: Vec<Literal>,
        source: Source,
        variables: &impl VariableStore,
    ) -> Self {
        let figured_out = figure_out_intial_watches(clause, variables);
        let stored_clause = Self {
            id,
            key,
            lbd: UnsafeCell::new(0),
            source,
            cached_a: figured_out[0],
            cached_b: figured_out[1],
            clause: figured_out,
            subsumed: vec![],
            node_index: None,
        };

        let watched_a = stored_clause.get_watched(Watch::A);
        let watched_b = stored_clause.get_watched(Watch::B);

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

    pub const fn source(&self) -> &Source {
        &self.source
    }

    pub fn get_watched(&self, watch: Watch) -> Literal {
        match watch {
            Watch::A => self.cached_a,
            Watch::B => self.cached_b,
        }
    }

    pub fn set_lbd(&self, variables: &impl VariableStore) {
        unsafe { *self.lbd.get() = self.lbd(variables) }
    }

    pub fn get_set_lbd(&self) -> usize {
        unsafe { *self.lbd.get() }
    }

    fn watch_update_replace(
        &mut self,
        watch: Watch,
        index: usize,
        variable: &Variable,
        literal: Literal,
    ) {
        let clause_index = match watch {
            Watch::A => {
                self.cached_a = literal;
                0
            }
            Watch::B => {
                self.cached_b = literal;
                1
            }
        };
        // let mix_up = index / 3;
        // if mix_up > 2 {
        //     self.clause.swap(index, mix_up);
        //     self.clause.swap(mix_up, clause_index);
        // } else {
        self.clause.swap(index, clause_index);
        // }

        variable.watch_added(self.key, literal.polarity());
    }

    pub fn update_watch(&mut self, watch: Watch, variables: &impl VariableStore) {
        'search_loop: for index in 2..self.clause.len() {
            let the_literal = unsafe { *self.clause.get_unchecked(index) };
            let the_variable = variables.get_unsafe(the_literal.index());

            match the_variable.polarity() {
                None => {
                    self.watch_update_replace(watch, index, the_variable, the_literal);
                    break 'search_loop;
                }
                Some(polarity) if polarity == the_literal.polarity() => {
                    self.watch_update_replace(watch, index, the_variable, the_literal);
                    break 'search_loop;
                }
                Some(_) => {}
            }
        }
    }

    /// 'Subsumes' a clause by removing the given literal.
    /// Records the clause has been subsumed, but does not store a record.
    /// In order to keep a record of the clauses used to prove the subsumption, use `literal_subsumption_core`.
    /// Returns Ok(()) if subsumption was ok, Err(()) otherwise
    pub fn literal_subsumption(
        &mut self,
        literal: Literal,
        variables: &impl VariableStore,
    ) -> Result<(), ()> {
        if self.clause.len() > 2 {
            if let Some(position) = self
                .clause
                .iter()
                .position(|clause_literal| *clause_literal == literal)
            {
                let last = *self.clause.last().expect("literally last");
                let removed = self.clause.swap_remove(position);
                if removed == self.cached_a {
                    self.cached_a = last;
                    self.update_watch(Watch::A, variables);
                } else if removed == self.cached_b {
                    self.cached_b = last;
                    self.update_watch(Watch::B, variables);
                }
                self.subsumed.push(removed);
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
        for hidden in &self.subsumed {
            original.push(*hidden)
        }
        original
    }

    pub fn id(&self) -> ClauseId {
        self.id
    }

    pub fn add_node_index(&mut self, index: NodeIndex) {
        self.node_index = Some(index);
    }

    pub fn get_node_index(&self) -> NodeIndex {
        self.node_index.expect("index requested but not set")
    }
}

impl std::fmt::Display for StoredClause {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.clause.as_string())
    }
}

fn figure_out_intial_watches(mut clause: Vec<Literal>, val: &impl VariableStore) -> Vec<Literal> {
    let length = clause.len();
    let mut watch_a = 0;
    let mut watch_b = 1;
    let mut a_status = get_status(unsafe { *clause.get_unchecked(watch_a) }, val);
    let mut b_status = get_status(unsafe { *clause.get_unchecked(watch_b) }, val);

    /*
    The initial setup gurantees a has status none or witness, while b may have any status
    priority is given to watch a, so that watch b remains a conflict until watch a becomes none
    at which point, b inherits the witness status of a (which may be updated again) or becomes none and no more checks need to happen
     */

    for index in 2..length {
        if a_status == WatchStatus::None && b_status == WatchStatus::None {
            break;
        }
        let literal = unsafe { *clause.get_unchecked(index) };
        let literal_status = get_status(literal, val);
        match literal_status {
            WatchStatus::Conflict => {
                // do nothing on a conflict
            }
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

fn get_status(literal: Literal, variables: &impl VariableStore) -> WatchStatus {
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
