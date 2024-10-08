use crate::structures::{
    clause::{Clause, ClauseVec},
    literal::Literal,
    solve::ClauseKey,
    valuation::{Valuation, ValuationVec},
    variable::{Variable, VariableId},
};

use std::cell::Cell;

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
    watch_clause: Cell<ClauseVec>,
    watch_a: Cell<usize>,
    watch_b: Cell<usize>,
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
    SameConflict,
    SameImplication,
    SameSatisfied,
    NewImplication,
    NewSatisfied,
    NewTwoNone,
}

/// The value is used to suggest an updated index
#[derive(Debug)]
pub enum WatchUpdateEnum {
    Witness(usize),
    None(usize),
    No,
}

impl StoredClause {
    pub fn new_from(key: ClauseKey, clause: ClauseVec, source: ClauseSource) -> StoredClause {
        clause.is_empty().then(|| panic!("An empty clause"));

        StoredClause {
            key,
            lbd: Cell::new(0),
            source,
            clause: clause.clone(),
            watch_clause: Cell::from(clause),
            watch_a: Cell::from(0),
            watch_b: Cell::from(0),
        }
    }

    pub fn source(&self) -> &ClauseSource {
        &self.source
    }

    pub fn literal_at(&self, position: usize) -> Literal {
        unsafe { *self.clause.get_unchecked(position) }
    }

    pub fn get_watched(&self, a_or_b: Watch) -> Literal {
        unsafe {
            match a_or_b {
                Watch::A => *self.clause.get_unchecked(self.watch_a.get()),
                Watch::B => *self.clause.get_unchecked(self.watch_b.get()),
            }
        }
    }

    pub fn set_watch(&self, watch: Watch, index: usize) {
        match watch {
            Watch::A => self.watch_a.set(index),
            Watch::B => self.watch_b.set(index),
        }
    }

    /// Find the index of a literal which has not been valued, if possible, else if there was some witness for the clause, return that
    pub fn some_none_or_else_witness_idx(
        &self,
        val: &impl Valuation,
        but_not: Option<VariableId>,
    ) -> WatchUpdateEnum {
        let mut witness = None;

        for (idx, literal) in self.clause.iter().enumerate() {
            if but_not.is_none() || but_not.is_some_and(|exclude| literal.v_id != exclude) {
                match val.of_v_id(literal.v_id) {
                    None => return WatchUpdateEnum::None(idx),
                    Some(value) => {
                        if value == literal.polarity {
                            witness = Some(idx)
                        }
                    }
                }
            }
        }
        match witness {
            Some(idx) => WatchUpdateEnum::Witness(idx),
            None => WatchUpdateEnum::No,
        }
    }

    pub fn watch_status(&self, val: &impl Valuation) -> ClauseStatus {
        let a_literal = self.clause[self.watch_a.get()];
        let a_val = val.of_v_id(a_literal.v_id);

        match self.clause.len() {
            1 => match a_val {
                // both watches point to the only literal
                Some(polarity) if polarity == a_literal.polarity => ClauseStatus::Satisfied,
                Some(_) => ClauseStatus::Conflict,
                None => ClauseStatus::Entails(a_literal),
            },
            _ => {
                let b_literal = self.clause[self.watch_b.get()];
                let b_val = val.of_v_id(b_literal.v_id);

                match (a_val, b_val) {
                    (None, None) => ClauseStatus::Unsatisfied,
                    (Some(a), Some(b)) if a == a_literal.polarity || b == b_literal.polarity => {
                        ClauseStatus::Satisfied
                    }
                    (Some(a), None) if a == a_literal.polarity => ClauseStatus::Satisfied,
                    (Some(_), None) => ClauseStatus::Entails(b_literal),
                    (None, Some(b)) if b == b_literal.polarity => ClauseStatus::Satisfied,
                    (None, Some(_)) => ClauseStatus::Entails(a_literal),
                    (Some(_), Some(_)) => ClauseStatus::Conflict,
                }
            }
        }
    }

    pub fn set_lbd(&self, vars: &[Variable]) {
        self.lbd.set(self.clause.lbd(vars));
    }

    pub fn lbd(&self) -> usize {
        self.lbd.get()
    }

    pub fn literal_of(&self, watch: Watch) -> Literal {
        match watch {
            Watch::A => self.clause[self.watch_a.get()],
            Watch::B => self.clause[self.watch_b.get()],
        }
    }

    pub fn clause_clone(&self) -> ClauseVec {
        self.clause.clone()
    }
}

/// Initialises the watches for a stored clause, is not a method as requires pointer information
pub fn initialise_watches_for(
    stored_clause: &StoredClause,
    val: &impl Valuation,
    vars: &[Variable],
) {
    if stored_clause.clause.len() > 1 {
        match stored_clause.some_none_or_else_witness_idx(val, None) {
            WatchUpdateEnum::Witness(index) | WatchUpdateEnum::None(index) => {
                stored_clause.watch_a.set(index)
            }
            WatchUpdateEnum::No => {
                panic!("n");
            }
        }

        let literal_a = stored_clause.clause[stored_clause.watch_a.get()];
        match stored_clause.some_none_or_else_witness_idx(val, Some(literal_a.v_id)) {
            WatchUpdateEnum::Witness(index) | WatchUpdateEnum::None(index) => {
                stored_clause.watch_b.set(index)
            }
            WatchUpdateEnum::No => {
                if stored_clause.watch_a.get() == 0 {
                    stored_clause.watch_b.set(1)
                } else {
                    stored_clause.watch_b.set(0)
                }
            }
        }

        let current_a = stored_clause.clause[stored_clause.watch_a.get()];
        vars[current_a.v_id].watch_added(stored_clause.key, current_a.polarity);

        let current_b = stored_clause.clause[stored_clause.watch_b.get()];
        vars[current_b.v_id].watch_added(stored_clause.key, current_b.polarity);
    } else {
        let watched_variable = stored_clause.clause.first().unwrap();
        vars[watched_variable.v_id].watch_added(stored_clause.key, watched_variable.polarity);
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
