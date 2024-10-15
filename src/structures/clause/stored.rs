use crate::structures::{
    clause::Clause, literal::Literal, solve::store::ClauseKey, valuation::Valuation, variable::Variable,
};

use std::cell::UnsafeCell;
use std::ops::Deref;

pub struct StoredClause {
    key: ClauseKey,
    lbd: UnsafeCell<usize>,
    source: Source,
    clause: Vec<Literal>,
    cached_a: Literal,
    cached_b: Literal,
}

// { Clause enums

#[derive(Clone, Debug)]
pub enum Source {
    Formula,
    Resolution(Vec<ClauseKey>),
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
        key: ClauseKey,
        clause: Vec<Literal>,
        source: Source,
        valuation: &impl Valuation,
        variables: &mut [Variable],
    ) -> Self {
        let figured_out = figure_out_intial_watches(clause.clone(), valuation);
        let stored_clause = Self {
            key,
            lbd: UnsafeCell::new(0),
            source,
            clause: figured_out.clone(),
            cached_a: figured_out[0],
            cached_b: figured_out[1],
        };

        let watched_a = stored_clause.get_watched(Watch::A);
        let watched_b = stored_clause.get_watched(Watch::B);
        unsafe {
            variables
                .get_unchecked(watched_a.index())
                .watch_added(stored_clause.key, watched_a.polarity());

            variables
                .get_unchecked(watched_b.index())
                .watch_added(stored_clause.key, watched_b.polarity());
        }

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

    pub fn set_lbd(&self, vars: &[Variable]) {
        unsafe { *self.lbd.get() = self.lbd(vars) }
    }

    pub fn get_set_lbd(&self) -> usize {
        unsafe { *self.lbd.get() }
    }

    fn watch_update_replace(
        &mut self,
        watch: Watch,
        index: usize,
        variables: &[Variable],
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
        let mix_up = index / 3;
        if mix_up > 2 {
            self.clause.swap(index, mix_up);
            self.clause.swap(mix_up, clause_index);
        } else {
            self.clause.swap(index, clause_index);
        }

        unsafe {
            variables
                .get_unchecked(literal.index())
                .watch_added(self.key(), literal.polarity());
        };
    }

    pub fn update_watch(
        &mut self,
        watch: Watch,
        valuation: &impl Valuation,
        variables: &[Variable],
    ) {
        'search_loop: for index in 2..self.clause.len() {
            let the_literal = unsafe { *self.clause.get_unchecked(index) };

            match valuation.of_index(the_literal.index()) {
                None => {
                    self.watch_update_replace(watch, index, variables, the_literal);
                    break 'search_loop;
                }
                Some(polarity) if polarity == the_literal.polarity() => {
                    self.watch_update_replace(watch, index, variables, the_literal);
                    break 'search_loop;
                }
                Some(_) => {}
            }
        }
    }
}

impl std::fmt::Display for StoredClause {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.clause.as_string())
    }
}

fn figure_out_intial_watches(clause: Vec<Literal>, val: &impl Valuation) -> Vec<Literal> {
    let length = clause.len();
    let mut the_wc = clause;
    let mut watch_a = 0;
    let mut watch_b = 1;
    let mut a_status = get_status(unsafe { *the_wc.get_unchecked(watch_a) }, val);
    let mut b_status = get_status(unsafe { *the_wc.get_unchecked(watch_b) }, val);

    /*
    The initial setup gurantees a has status none or witness, while b may have any status
    priority is given to watch a, so that watch b remains a conflict until watch a becomes none
    at which point, b inherits the witness status of a (which may be updated again) or becomes none and no more checks need to happen
     */

    for index in 2..length {
        if a_status == WatchStatus::None && b_status == WatchStatus::None {
            break;
        }
        let literal = unsafe { *the_wc.get_unchecked(index) };
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

        the_wc.swap(0, watch_a);
        the_wc.swap(1, watch_b);
    }

    the_wc
}

fn get_status(literal: Literal, valuation: &impl Valuation) -> WatchStatus {
    match valuation.of_index(literal.index()) {
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
