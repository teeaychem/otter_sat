use crate::{
    literal,
    structures::{Clause, ClauseId, ClauseVec, Literal},
    Valuation, VariableId,
};

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
    watch_a: usize,
    watch_b: usize,
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
            watch_a: 0,
            watch_b: 1,
        };

        the_clause.watch_a = the_clause
            .some_status_index(val, None)
            .expect("Could not set watch A");
        if let Some(index) = the_clause.some_status_index_ignoring(
            val,
            None,
            the_clause.clause[the_clause.watch_a].v_id,
        ) {
            the_clause.watch_b = index
        } else if let Some(index) = the_clause.some_status_index_ignoring(
            val,
            Some(true),
            the_clause.clause[the_clause.watch_a].v_id,
        ) {
            the_clause.watch_b = index
        } else if let Some(index) = the_clause.some_status_index_ignoring(
            val,
            Some(false),
            the_clause.clause[the_clause.watch_a].v_id,
        ) {
            the_clause.watch_b = index
        } else {
            panic!("Could not set watch B");
        };

        if the_clause.watch_a == the_clause.watch_b {
            panic!(
                "Same watch on new clause {} {}",
                the_clause.watch_a, the_clause.watch_b
            );
        }

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

    pub fn watch_status(&self, val: &impl Valuation) -> (Option<bool>, Option<bool>) {
        println!("Watch status of clause: {}", self.clause.as_string());
        println!("A: {}", self.watch_a);
        println!("B: {}", self.watch_b);

        let a_status = match val.of_v_id(self.clause[self.watch_a].v_id) {
            Ok(optional) => optional,
            _ => panic!("Watch literal without status"),
        };
        let b_status = match val.of_v_id(self.clause[self.watch_b].v_id) {
            Ok(optional) => optional,
            _ => panic!("Watch literal without status"),
        };

        (a_status, b_status)
    }

    fn some_status_index(&self, val: &impl Valuation, status: Option<bool>) -> Option<usize> {
        self.clause
            .iter()
            .enumerate()
            .find(|(_, l)| val.of_v_id(l.v_id).is_ok_and(|v| v == status))
            .map(|(idx, _)| idx)
    }

    fn some_status_index_ignoring(
        &self,
        val: &impl Valuation,
        status: Option<bool>,
        ignoring: VariableId,
    ) -> Option<usize> {
        self.clause
            .iter()
            .enumerate()
            .find(|(_, l)| l.v_id != ignoring && val.of_v_id(l.v_id).is_ok_and(|v| v == status))
            .map(|(idx, _)| idx)
    }

    fn index_of(&self, vid: VariableId) -> usize {
        self.clause
            .iter()
            .enumerate()
            .find(|(_, l)| l.v_id == vid)
            .map(|(idx, _)| idx)
            .expect("Literal not found in clause")
    }

    /// Updates the two watched literals on the assumption that only the valuation of the current literal has changed.
    pub fn update_watch(&mut self, val: &impl Valuation, vid: VariableId) {

        let valuation_polarity = val.of_v_id(vid).unwrap().unwrap();
        let clause_polarity = self
            .clause
            .literals()
            .find(|l| l.v_id == vid)
            .unwrap()
            .polarity;

        let current_a = self.clause[self.watch_a];
        let current_b = self.clause[self.watch_b];

        if valuation_polarity == clause_polarity {
            let index_of_literal = self.index_of(vid);
            if self.watch_a != index_of_literal && self.watch_b != index_of_literal {
                if Some(current_a.polarity) != val.of_v_id(current_a.v_id).unwrap() {
                    self.watch_a = index_of_literal
                } else if Some(current_b.polarity) != val.of_v_id(current_b.v_id).unwrap() {
                    self.watch_b = index_of_literal
                }
            }
        }
        if valuation_polarity != clause_polarity {
            if self.clause[self.watch_a].v_id == vid {
                if let Some(new_idx) = self.some_status_index_ignoring(val, None, current_b.v_id) {
                    // println!("Setting A ({}) to {}", self.watch_a, new_idx);
                    self.watch_a = new_idx
                };
            }
            if self.clause[self.watch_b].v_id == vid {
                if let Some(new_idx) = self.some_status_index_ignoring(val, None, current_a.v_id) {
                    // println!("Setting B ({}) to {}", self.watch_b, new_idx);
                    self.watch_b = new_idx
                };
            }
        }
    }

    pub fn watch_choices(&self, val: &impl Valuation) -> Option<Vec<Literal>> {
        let a_literal = self.clause[self.watch_a];
        let b_literal = self.clause[self.watch_b];

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
