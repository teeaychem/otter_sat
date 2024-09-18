use crate::{
    literal,
    structures::{Clause, ClauseId, ClauseVec, Literal},
    Valuation,
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
    pub fn new_from(id: ClauseId, clause: &impl Clause, source: ClauseSource) -> StoredClause {
        if clause.as_vec().len() < 2 {
            panic!("Short clause (≤ 1)")
        }

        StoredClause {
            id,
            clause: clause.as_vec(),
            source,
            watch_a: 0,
            watch_b: 1,
        }
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

    /// Returns the index of some literal whose value is not set on the given valuation
    fn some_none_index(&self, val: &impl Valuation) -> Option<usize> {
        self.clause
            .iter()
            .enumerate()
            .find(|(_, l)| val.of_v_id(l.v_id).is_ok_and(|v| v.is_none()))
            .map(|(idx, _)| idx)
    }

    /// Updates the two watched literals on the assumption that only the valuation of the current literal has changed.
    pub fn update_watch(&mut self, val: &impl Valuation, lit: Literal) {
        let watch_status = self.watch_status(val);
        match lit.polarity {
            true => match watch_status {
                (Some(true), _) | (_, Some(true)) => {
                    println!("Watch is already true, so nothing to do…")
                }
                (Some(false), Some(false)) => {
                    println!("Watch is false, so…");
                    self.watch_a = lit.v_id;
                }
                (None, _) => {
                    self.watch_a = lit.v_id;
                }
                (_, None) => {
                    self.watch_b = lit.v_id;
                }
            },
            false => match watch_status {
                (Some(false), Some(false)) => {
                    if self.clause[self.watch_a].v_id == lit.v_id {
                        if let Some(new_idx) = self.some_none_index(val) {
                            self.watch_a = new_idx
                        };
                    } else if self.clause[self.watch_b].v_id == lit.v_id {
                        if let Some(new_idx) = self.some_none_index(val) {
                            self.watch_b = new_idx
                        };
                    } else {
                        // there is nothing to be done as all other literals must be false
                    }
                }
                _ => panic!("Nothing to do"),
            },
        }
    }
}

impl std::fmt::Display for StoredClause {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "#[{}] {}", self.id, self.clause.as_string())
    }
}
