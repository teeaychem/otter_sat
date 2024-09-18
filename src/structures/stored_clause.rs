use crate::structures::{Clause, ClauseId, Literal};

#[derive(Clone, Debug)]
pub enum ClauseSource {
    Formula,
    Resolution,
}

#[derive(Clone, Debug)]
pub struct StoredClause {
    pub id: ClauseId,
    pub source: ClauseSource,
    pub clause: Vec<Literal>,
}

impl std::fmt::Display for StoredClause {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "#[{}] ", self.id)?;
        write!(f, "(")?;
        for literal in self.clause.iter() {
            write!(f, " {literal} ")?;
        }
        write!(f, ")")?;
        Ok(())
    }
}

impl StoredClause {
    pub fn new_from(id: ClauseId, clause: &impl Clause, source: ClauseSource) -> StoredClause {
        StoredClause {
            id,
            clause: clause.as_vec(),
            source,
        }
    }
}
