use crate::{
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
}

impl std::fmt::Display for StoredClause {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "#[{}] {}", self.id, self.clause.as_string())
    }
}
