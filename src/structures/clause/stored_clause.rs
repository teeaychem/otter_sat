use crate::structures::{
    clause::{Clause, ClauseVec},
    literal::Literal,
    solve::ClauseKey,
    valuation::{Valuation, ValuationVec},
    variable::{Variable, VariableId},
};

use std::cell::UnsafeCell;

pub struct StoredClause {
    key: ClauseKey,
    lbd: UnsafeCell<usize>,
    source: ClauseSource,
    clause: ClauseVec,
    the_wc: ClauseVec,
    cached_split_watches: ((VariableId, bool), (VariableId, bool)),
}

// { Clause enums

#[derive(Debug)]
pub enum ClauseStatus {
    Satisfied,        // some watch literal matches
    Conflict,         // no watch literals matches
    Implies(Literal), // Literal is unassigned and the no other watch matches
    Unsatisfied,      // more than one literal is unassigned
    Missing,          // it's been removed, useful for lazy lists
}

#[derive(Clone, Debug)]
pub enum ClauseSource {
    Formula,
    Resolution(Vec<ClauseKey>),
}

// }

// { Watch enums

#[derive(Debug, Clone, Copy)]
pub enum Watch {
    A,
    B,
}

#[derive(Clone, Copy, PartialEq)]
pub enum WatchUpdate {
    NoUpdate,
    Update(Literal),
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
        clause: ClauseVec,
        source: ClauseSource,
        valuation: &impl Valuation,
        variables: &mut [Variable],
    ) -> StoredClause {
        let figured_out = figure_out_intial_watches(clause.clone(), valuation);
        let stored_clause = StoredClause {
            key,
            lbd: UnsafeCell::new(0),
            source,
            clause,
            the_wc: figured_out.clone(),
            cached_split_watches: (
                (figured_out[0].v_id, figured_out[0].polarity),
                (figured_out[1].v_id, figured_out[1].polarity),
            ),
        };

        unsafe {
            let (current_a_v_id, current_a_polarity) = stored_clause.get_watched_split(Watch::A);
            variables
                .get_unchecked(current_a_v_id as usize)
                .watch_added(stored_clause.key, current_a_polarity);

            let (current_b_v_id, current_b_polarity) = stored_clause.get_watched_split(Watch::B);
            variables
                .get_unchecked(current_b_v_id as usize)
                .watch_added(stored_clause.key, current_b_polarity);
        }

        stored_clause
    }

    pub fn key(&self) -> ClauseKey {
        self.key
    }

    pub fn source(&self) -> &ClauseSource {
        &self.source
    }

    pub fn literal_at(&self, position: usize) -> Literal {
        unsafe { *self.clause.get_unchecked(position) }
    }

    pub fn get_watched_split(&self, watch: Watch) -> (VariableId, bool) {
        match watch {
            Watch::A => self.cached_split_watches.0,
            Watch::B => self.cached_split_watches.1,
        }
    }

    pub fn get_watched_v_id(&self, watch: Watch) -> VariableId {
        match watch {
            Watch::A => self.cached_split_watches.0 .0,
            Watch::B => self.cached_split_watches.1 .0,
        }
    }

    pub fn watch_status(&self, val: &impl Valuation, id: VariableId) -> ClauseStatus {
        let (a_v_id, a_polarity) = self.get_watched_split(Watch::A);
        let (b_v_id, b_polarity) = self.get_watched_split(Watch::B);

        if a_v_id != id && b_v_id != id {
            return ClauseStatus::Missing;
        }

        match (val.of_v_id(a_v_id), val.of_v_id(b_v_id)) {
            (None, None) => ClauseStatus::Unsatisfied,
            (Some(a), None) if a == a_polarity => ClauseStatus::Satisfied,
            (None, Some(b)) if b == b_polarity => ClauseStatus::Satisfied,
            (Some(_), None) => ClauseStatus::Implies(Literal {
                v_id: b_v_id,
                polarity: b_polarity,
            }),
            (None, Some(_)) => ClauseStatus::Implies(Literal {
                v_id: a_v_id,
                polarity: a_polarity,
            }),
            (Some(a), Some(b)) if a == a_polarity || b == b_polarity => ClauseStatus::Satisfied,
            (Some(_), Some(_)) => ClauseStatus::Conflict,
        }
    }

    pub fn set_lbd(&self, vars: &[Variable]) {
        unsafe {
            *self.lbd.get() = self.lbd(vars);
        }
    }

    pub fn get_set_lbd(&self) -> usize {
        unsafe { *self.lbd.get() }
    }

    pub fn clause_clone(&self) -> ClauseVec {
        self.clause.clone()
    }

    pub fn update_watch(&mut self, watch: Watch, valuation: &impl Valuation) -> WatchUpdate {
        let mut replacement = WatchUpdate::NoUpdate;

        let length = self.the_wc.len();
        let mut index = 2;
        'search_loop: for _ in 2..length {
            let the_literal = unsafe { *self.the_wc.get_unchecked(index) };
            match get_status(the_literal, valuation) {
                WatchStatus::None | WatchStatus::Witness => {
                    replacement = WatchUpdate::Update(the_literal);
                    break 'search_loop;
                }
                WatchStatus::Conflict => {
                    index += 1;
                }
            }
        }

        match replacement {
            WatchUpdate::Update(the_literal) => {
                let clause_index = match watch {
                    Watch::A => {
                        self.cached_split_watches.0 .0 = the_literal.v_id;
                        self.cached_split_watches.0 .1 = the_literal.polarity;
                        0
                    }
                    Watch::B => {
                        self.cached_split_watches.1 .0 = the_literal.v_id;
                        self.cached_split_watches.1 .1 = the_literal.polarity;
                        1
                    }
                };
                let mix_up = index / 4;
                if mix_up > 2 {
                    self.the_wc.swap(mix_up, index);
                    self.the_wc.swap(clause_index, mix_up);
                } else {
                    self.the_wc.swap(clause_index, index);
                }
            }
            WatchUpdate::NoUpdate => {}
        }

        replacement
    }
}

impl std::fmt::Display for StoredClause {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.clause.as_string())
    }
}

/// Lift the method from the clause stored to the stored clause
impl Clause for StoredClause {
    fn literals(&self) -> impl Iterator<Item = Literal> {
        self.clause.literals()
    }

    fn variables(&self) -> impl Iterator<Item = VariableId> {
        self.clause.variables()
    }

    fn is_sat_on(&self, valuation: &ValuationVec) -> bool {
        self.clause.is_sat_on(valuation)
    }

    fn is_unsat_on(&self, valuation: &ValuationVec) -> bool {
        self.clause.is_unsat_on(valuation)
    }

    fn find_unit_literal<T: Valuation>(&self, valuation: &T) -> Option<Literal> {
        self.clause.find_unit_literal(valuation)
    }

    fn collect_choices<T: Valuation>(&self, valuation: &T) -> Option<Vec<Literal>> {
        self.clause.collect_choices(valuation)
    }

    fn as_string(&self) -> String {
        self.clause.as_string()
    }

    fn as_dimacs(&self, variables: &[Variable]) -> String {
        self.clause.as_dimacs(variables)
    }

    fn is_empty(&self) -> bool {
        self.clause.is_empty()
    }

    fn to_vec(self) -> ClauseVec {
        self.clause
    }

    fn length(&self) -> usize {
        self.clause.len()
    }

    fn asserts(&self, val: &impl Valuation) -> Option<Literal> {
        self.clause.asserts(val)
    }

    fn lbd(&self, variables: &[Variable]) -> usize {
        self.clause.lbd(variables)
    }

    fn find_literal_by_id(&self, id: VariableId) -> Option<Literal> {
        self.clause.find_literal_by_id(id)
    }
}

fn figure_out_intial_watches(clause: ClauseVec, val: &impl Valuation) -> Vec<Literal> {
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
        } else {
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
        }
        the_wc.swap(0, watch_a);
        the_wc.swap(1, watch_b);
    }

    the_wc
}

fn get_status(literal: Literal, valuation: &impl Valuation) -> WatchStatus {
    match valuation.of_v_id(literal.v_id) {
        None => WatchStatus::None,
        Some(polarity) if polarity == literal.polarity => WatchStatus::Witness,
        Some(_) => WatchStatus::Conflict,
    }
}
