use crate::structures::{
    clause::{Clause, ClauseVec},
    literal::Literal,
    solve::ClauseKey,
    valuation::{Valuation, ValuationVec},
    variable::{Variable, VariableId},
};

use orx_linked_list::{DoublyIterable, DoublyList};
use std::cell::Cell;

#[derive(Debug)]
pub struct WatchClause {
    watch_a: Literal,
    watch_b: Literal,
    the_rest: DoublyList<Literal>,
}

#[derive(Clone, Copy, PartialEq)]
enum Status {
    Witness,
    None,
    Conflict,
}

#[derive(Clone, Copy, PartialEq)]
pub enum WatchUpdate {
    NoUpdate,
    FromTo(Literal, Literal),
}

fn get_status(l: Literal, v: &impl Valuation) -> Status {
    match v.of_v_id(l.v_id) {
        None => Status::None,
        Some(polarity) if polarity == l.polarity => Status::Witness,
        Some(_) => Status::Conflict,
    }
}

impl WatchClause {
    pub fn new(clause: ClauseVec, val: &impl Valuation) -> Self {
        unsafe {
            let mut the_rest = DoublyList::new();
            let mut watch_a = *clause.get_unchecked(0);
            let mut watch_b = *clause.get_unchecked(1);
            let mut a_status = get_status(watch_a, val);
            let mut b_status = get_status(watch_b, val);

            /*
            The initial setup gurantees a has status none or witness, while b may have any status
            priority is given to watch a, so that watch b remains a conflict until watch a becomes none
            at which point, b inherits the witness status of a (which may be updated again) or becomes none and no more checks need to happen
             */

            for index in 2..clause.len() {
                let literal = *clause.get_unchecked(index);
                if a_status == Status::None && b_status == Status::None {
                    the_rest.push_back(literal);
                } else {
                    let literal_status = get_status(literal, val);
                    match literal_status {
                        Status::Conflict => {
                            // do nothing on a conflict
                            the_rest.push_back(literal);
                        }
                        Status::None => {
                            // by the first check, either a or b fails to be none, so update a or otherwise b
                            if a_status != Status::None {
                                // though, if a is acting as a witness, pass this to b
                                if a_status == Status::Witness {
                                    the_rest.push_back(watch_b);
                                    watch_b = watch_a;
                                    watch_a = literal;
                                    a_status = Status::None;
                                    b_status = Status::Witness;
                                } else {
                                    the_rest.push_back(watch_a);
                                    watch_a = literal;
                                    a_status = Status::None;
                                }
                            } else {
                                the_rest.push_back(watch_b);
                                watch_b = literal;
                                b_status = Status::None;
                            }
                        }
                        Status::Witness => {
                            if a_status == Status::Conflict {
                                the_rest.push_back(watch_a);
                                watch_a = literal;
                                a_status = Status::Witness;
                            } else {
                                the_rest.push_back(literal);
                            }
                        }
                    }
                }
            }

            WatchClause {
                watch_a,
                watch_b,
                the_rest,
            }
        }
    }

    pub fn update(&mut self, watch: Watch, valuation: &impl Valuation) -> WatchUpdate {
        let mut replacement = None;
        let mut witness = false;

        'search_loop: for idx in self.the_rest.indices() {
            let literal_at_index = self.the_rest[&idx];
            match get_status(literal_at_index, valuation) {
                Status::None => {
                    replacement = Some(idx);
                    break 'search_loop;
                }
                Status::Witness if !witness => {
                    replacement = Some(idx);
                    witness = true;
                }
                Status::Witness => {}
                Status::Conflict => {}
            }
        }
        if let Some(idx) = replacement {
            let new_watch = self.the_rest.remove(&idx);
            match watch {
                Watch::A => {
                    self.the_rest.push_front(self.watch_a);
                    let from = self.watch_a;
                    self.watch_a = new_watch;
                    WatchUpdate::FromTo(from, new_watch)
                }
                Watch::B => {
                    self.the_rest.push_front(self.watch_b);
                    let from = self.watch_b;
                    self.watch_b = new_watch;
                    WatchUpdate::FromTo(from, new_watch)
                }
            }
        } else {
            WatchUpdate::NoUpdate
        }
    }
}

#[derive(Clone, Debug)]
pub enum ClauseSource {
    Formula,
    Resolution(Vec<ClauseKey>),
}

#[derive(Debug, Clone, Copy)]
pub enum Watch {
    A,
    B,
}

/**
The stored clause struct associates a clause with metadata relevant for a solve
and, is intended to be the unique representation of a clause within a solve
- `lbd` is the literal block distance of the clause
  - note, this defaults to 0 and should be updated if a clause is stored after some decisions have been made
- `watch_a` and `watch_b` are pointers to the watched literals, and rely on a vector representation of the clause
  - note, both default to 0 and should be initialised with `initialise_watches_for` when the clause is stored
*/
pub struct StoredClause {
    pub key: ClauseKey,
    lbd: Cell<usize>,
    source: ClauseSource,
    clause: ClauseVec,
    pub watch_clause: WatchClause,
}

#[derive(Debug)]
pub enum ClauseStatus {
    Satisfied,        // some watch literal matches
    Conflict,         // no watch literals matches
    Entails(Literal), // Literal is unassigned and the no other watch matches
    Unsatisfied,      // more than one literal is unassigned
}

/// The same/new variants allow for contrast with a known previous state
#[derive(PartialEq, Debug)]
pub enum WatchStatus {
    Same,
    New,
}

/// The value is used to suggest an updated index
#[derive(Debug)]
pub enum WatchUpdateEnum {
    Witness(usize),
    None(usize),
    No,
}

impl StoredClause {
    pub fn new_from(
        key: ClauseKey,
        clause: ClauseVec,
        source: ClauseSource,
        valuation: &impl Valuation,
        variables: &mut [Variable],
    ) -> StoredClause {
        if clause.len() < 2 {
            panic!("Storing a short clause")
        }

        let stored_clause = StoredClause {
            key,
            lbd: Cell::new(0),
            source,
            clause: clause.clone(),
            watch_clause: WatchClause::new(clause, valuation),
        };

        unsafe {
            let current_a = stored_clause.watch_clause.watch_a;

            variables
                .get_unchecked(current_a.v_id)
                .watch_added(stored_clause.key, current_a.polarity);

            let current_b = stored_clause.watch_clause.watch_b;

            variables
                .get_unchecked(current_b.v_id)
                .watch_added(stored_clause.key, current_b.polarity);
        }

        stored_clause
    }

    pub fn source(&self) -> &ClauseSource {
        &self.source
    }

    pub fn literal_at(&self, position: usize) -> Literal {
        unsafe { *self.clause.get_unchecked(position) }
    }

    pub fn get_watched(&self, a_or_b: Watch) -> Literal {
        match a_or_b {
            Watch::A => self.watch_clause.watch_a,
            Watch::B => self.watch_clause.watch_b,
        }
    }

    pub fn watch_status(&self, val: &impl Valuation) -> ClauseStatus {
        let a_literal = self.watch_clause.watch_a;
        let a_val = val.of_v_id(a_literal.v_id);

        match self.clause.len() {
            1 => match a_val {
                // both watches point to the only literal
                Some(polarity) if polarity == a_literal.polarity => ClauseStatus::Satisfied,
                Some(_) => ClauseStatus::Conflict,
                None => ClauseStatus::Entails(a_literal),
            },
            _ => {
                let b_literal = self.watch_clause.watch_b;
                let b_val = val.of_v_id(b_literal.v_id);

                match (a_val, b_val) {
                    (None, None) => ClauseStatus::Unsatisfied,
                    (Some(a), None) if a == a_literal.polarity => ClauseStatus::Satisfied,
                    (Some(_), None) => ClauseStatus::Entails(b_literal),
                    (None, Some(b)) if b == b_literal.polarity => ClauseStatus::Satisfied,
                    (None, Some(_)) => ClauseStatus::Entails(a_literal),
                    (Some(a), Some(b)) if a == a_literal.polarity || b == b_literal.polarity => {
                        ClauseStatus::Satisfied
                    }
                    (Some(_), Some(_)) => ClauseStatus::Conflict,
                }
            }
        }
    }

    pub fn set_lbd(&self, vars: &[Variable]) {
        self.lbd.set(self.lbd(vars));
    }

    pub fn get_set_lbd(&self) -> usize {
        self.lbd.get()
    }

    pub fn literal_of(&self, watch: Watch) -> Literal {
        match watch {
            Watch::A => self.watch_clause.watch_a,
            Watch::B => self.watch_clause.watch_b,
        }
    }

    pub fn clause_clone(&self) -> ClauseVec {
        self.clause.clone()
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
