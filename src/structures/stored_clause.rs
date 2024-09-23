use crate::structures::{Clause, ClauseId, ClauseVec, Literal, Valuation, VariableId};

use std::{borrow::Borrow, cell::Cell};

#[derive(Clone, Copy, Debug)]
pub enum ClauseSource {
    Formula,
    Resolution,
}

#[derive(Clone, Debug)]
pub struct StoredClause {
    id: ClauseId,
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
        val: &impl Valuation,
    ) -> StoredClause {
        if clause.as_vec().len() < 2 {
            panic!("Short clause (≤ 1)")
        }

        let mut the_clause = StoredClause {
            id,
            clause: clause.as_vec(),
            source,
            watch_a: 0.into(),
            watch_b: 1.into(),
        };

        the_clause.watch_a = the_clause.some_preferred_index(val, None).into();

        the_clause.watch_b = {
            let literal_a = the_clause.clause[the_clause.watch_a.get()];
            the_clause
                .some_preferred_index(val, Some(literal_a.v_id))
                .into()
        };

        the_clause
    }

    pub fn id(&self) -> ClauseId {
        self.id
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
                excluded && val.of_v_id(l.v_id).unwrap().is_none()
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
                let polarity_match = if let Some(v) = val.of_v_id(l.v_id).unwrap() {
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
                let polarity_match = if let Some(v) = val.of_v_id(l.v_id).unwrap() {
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
        let current_a = self.clause[self.watch_a.get()];
        let current_b = self.clause[self.watch_b.get()];

        let polarity_match = {
            let valuation_polarity = val.of_v_id(v_id).unwrap().unwrap();
            let clause_polarity = self.literals().find(|l| l.v_id == v_id).unwrap().polarity;
            valuation_polarity == clause_polarity
        };

        if polarity_match {
            let index_of_literal = self.index_of(v_id);
            if self.watch_a.get() != index_of_literal && self.watch_b.get() != index_of_literal {
                if Some(current_a.polarity) != val.of_v_id(current_a.v_id).unwrap() {
                    self.watch_a.replace(index_of_literal);
                } else if Some(current_b.polarity) != val.of_v_id(current_b.v_id).unwrap() {
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
        let current_a_match = Some(current_a.polarity) == val.of_v_id(current_a.v_id).unwrap();
        let current_b = self.clause[self.watch_b.get()];
        let current_b_match = Some(current_b.polarity) == val.of_v_id(current_b.v_id).unwrap();

        !(current_a_match || current_b_match)
    }

    pub fn watch_choices(&self, val: &impl Valuation) -> Option<Vec<Literal>> {
        let a_literal = self.clause[self.watch_a.get()];
        let b_literal = self.clause[self.watch_b.get()];

        let a_val = val.of_v_id(a_literal.v_id);
        let b_val = val.of_v_id(b_literal.v_id);

        let mut the_vec = vec![];

        if !(Ok(Some(a_literal.polarity)) == a_val || Ok(Some(b_literal.polarity)) == b_val) {
            if Ok(None) == a_val {
                the_vec.push(a_literal)
            }
            if Ok(None) == b_val {
                the_vec.push(b_literal)
            }
            Some(the_vec)
        } else {
            None
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
