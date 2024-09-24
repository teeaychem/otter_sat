use crate::structures::{Clause, ClauseId, ClauseVec, Literal, Valuation, VariableId};
use petgraph::matrix_graph::Zero;
use petgraph::prelude::NodeIndex;

use std::cell::{Cell, OnceCell};
use std::rc::Rc;

#[derive(Clone, Copy, Debug)]
pub enum ClauseSource {
    Formula,
    Resolution,
}

#[derive(Clone, Debug)]
pub struct StoredClause {
    id: ClauseId,
    nx: OnceCell<NodeIndex>,
    source: ClauseSource,
    clause: ClauseVec,
    watch_a: Cell<usize>,
    watch_b: Cell<usize>,
}

/*
A stored clause is implicitly tied to the status of a solve via the two watches.
- If any literal in the watch is true on the current valuation of the solve, then one of the watch literals will be set to Some(true)
- Both watch literals will be set to Some(false)  only  if  it is not possible to find a literal with a value other than Some(false) on the current valuation
 */
impl StoredClause {
    pub fn new_from(
        id: ClauseId,
        clause: &impl Clause,
        source: ClauseSource,
    ) -> Rc<StoredClause> {
        if clause.as_vec().len().is_zero() {
            panic!("An empty clause")
        }

        let the_clause = StoredClause {
            id,
            nx: OnceCell::new(),
            clause: clause.as_vec(),
            source,
            watch_a: 0.into(),
            watch_b: 0.into(),
        };

        Rc::new(the_clause)
    }

    pub fn id(&self) -> ClauseId {
        self.id
    }

    pub fn initialise_watches_for(&self, val: &impl Valuation) {
        if self.clause.len() > 1 {
            self.watch_a.replace(self.some_preferred_index(val, None));

            self.watch_b.replace({
                let literal_a = self.clause[self.watch_a.get()];
                self.some_preferred_index(val, Some(literal_a.v_id))
            });
        }
    }

    pub fn nx(&self) -> NodeIndex {
        match self.nx.get() {
            None => panic!("Attempt to access resolution node index before it has been set"),
            Some(&x) => x,
        }
    }

    pub fn set_nx(&self, nx: NodeIndex) {
        let _ = self.nx.set(nx);
    }

    pub fn source(&self) -> ClauseSource {
        self.source
    }

    pub fn clause(&self) -> &impl Clause {
        &self.clause
    }

    pub fn literals(&self) -> impl Iterator<Item = Literal> + '_ {
        self.clause.literals()
    }

    fn index_of(&self, vid: VariableId) -> usize {
        self.clause
            .iter()
            .enumerate()
            .find(|(_, l)| l.v_id == vid)
            .map(|(idx, _)| idx)
            .expect("Literal not found in clause")
    }
}

impl StoredClause {
    /// Finds an index of the clause vec whose value is None on val and differs from but_not.
    fn some_none_index(&self, val: &impl Valuation, but_not: Option<VariableId>) -> Option<usize> {
        self.clause
            .iter()
            .enumerate()
            .find(|(_, l)| {
                let excluded = if let Some(to_exclude) = but_not {
                    l.v_id != to_exclude
                } else {
                    true
                };
                excluded && val.of_v_id(l.v_id).is_none()
            })
            .map(|(idx, _)| idx)
    }

    /// Finds an index of the clause vec which witness the clause is true on val and differs from but_not.
    fn some_witness_index(
        &self,
        val: &impl Valuation,
        but_not: Option<VariableId>,
    ) -> Option<usize> {
        self.clause
            .iter()
            .enumerate()
            .find(|(_, l)| {
                let excluded = if let Some(to_exclude) = but_not {
                    l.v_id != to_exclude
                } else {
                    true
                };
                let polarity_match = if let Some(v) = val.of_v_id(l.v_id) {
                    v == l.polarity
                } else {
                    false
                };
                excluded && polarity_match
            })
            .map(|(idx, _)| idx)
    }

    /// Finds an index of the clause vec which witness the clause is false on val and differs from but_not.
    fn some_differing_index(
        &self,
        val: &impl Valuation,
        but_not: Option<VariableId>,
    ) -> Option<usize> {
        self.clause
            .iter()
            .enumerate()
            .find(|(_, l)| {
                let excluded = if let Some(to_exclude) = but_not {
                    l.v_id != to_exclude
                } else {
                    true
                };
                let polarity_match = if let Some(v) = val.of_v_id(l.v_id) {
                    v != l.polarity
                } else {
                    false
                };
                excluded && polarity_match
            })
            .map(|(idx, _)| idx)
    }

    /// Finds some index of the clause vec which isn't but_not with the preference:
    ///   A. The index points to a literal which is true on val.
    ///   B. The index points to a literal which is unassigned on val.
    ///   C. The index points to a literal which is false on val.
    /// This preference contributes to maintaining useful watch literals.
    /// As, it is essentail to know when a clause is true, as it then can provide no useful information.
    /// And, if a watch is only on a differing literal when there are no other unassigned literals
    /// it follows the other watched literal must be true on the valuation, or else there's a contradiction.
    fn some_preferred_index(&self, val: &impl Valuation, but_not: Option<usize>) -> usize {
        if let Some(index) = self.some_witness_index(val, but_not) {
            index
        } else if let Some(index) = self.some_none_index(val, but_not) {
            index
        } else if let Some(index) = self.some_differing_index(val, but_not) {
            index
        } else {
            panic!("Could not find a suitable index");
        }
    }

    /// Updates the two watched literals on the assumption that only the valuation of the given id has changed.
    /// Return true if the watch is 'informative' (the clause is unit or conflicts on val)
    pub fn update_watch(&self, val: &impl Valuation, v_id: VariableId) -> bool {
        if self.clause.len() == 1 {
            return val.of_v_id(self.clause[self.watch_a.get()].v_id).is_none();
        }

        let current_a = self.clause[self.watch_a.get()];
        let current_b = self.clause[self.watch_b.get()];

        let polarity_match = {
            let valuation_polarity = val.of_v_id(v_id).unwrap();
            let clause_polarity = self.literals().find(|l| l.v_id == v_id).unwrap().polarity;
            valuation_polarity == clause_polarity
        };

        if polarity_match {
            let index_of_literal = self.index_of(v_id);
            if self.watch_a.get() != index_of_literal && self.watch_b.get() != index_of_literal {
                if Some(current_a.polarity) != val.of_v_id(current_a.v_id) {
                    self.watch_a.replace(index_of_literal);
                } else if Some(current_b.polarity) != val.of_v_id(current_b.v_id) {
                    self.watch_b.replace(index_of_literal);
                }
            }
        } else if current_a.v_id == v_id {
            if let Some(new_idx) = self.some_none_index(val, Some(current_b.v_id)) {
                self.watch_a.replace(new_idx);
            };
        } else if current_b.v_id == v_id {
            if let Some(new_idx) = self.some_none_index(val, Some(current_a.v_id)) {
                self.watch_b.replace(new_idx);
            };
        }

        let current_a = self.clause[self.watch_a.get()];
        let current_a_match = Some(current_a.polarity) == val.of_v_id(current_a.v_id);
        let current_b = self.clause[self.watch_b.get()];
        let current_b_match = Some(current_b.polarity) == val.of_v_id(current_b.v_id);

        !(current_a_match || current_b_match)
    }
}

pub enum ClauseStatus {
    Satisfied,        // some watch literal matches
    Conflict,         // no watch literals matches
    Entails(Literal), // Literal is unassigned and the no other watch matches
    Unsatisfied,      // more than one literal is unassigned
}

impl StoredClause {
    pub fn watch_choices(&self, val: &impl Valuation) -> ClauseStatus {
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

                if a_val.is_none() && b_val.is_none() {
                    ClauseStatus::Unsatisfied
                } else if a_val.is_none() && Some(b_literal.polarity) != b_val {
                    ClauseStatus::Entails(a_literal)
                } else if b_val.is_none() && Some(a_literal.polarity) != a_val {
                    ClauseStatus::Entails(b_literal)
                } else if Some(a_literal.polarity) == a_val || Some(b_literal.polarity) == b_val {
                    ClauseStatus::Satisfied
                } else {
                    ClauseStatus::Conflict
                }
            }
        }
    }
}

impl std::fmt::Display for StoredClause {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "#[{}] {}", self.id, self.clause.as_string())
    }
}

impl PartialOrd for StoredClause {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for StoredClause {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.id.cmp(&other.id)
    }
}

impl PartialEq for StoredClause {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for StoredClause {}
