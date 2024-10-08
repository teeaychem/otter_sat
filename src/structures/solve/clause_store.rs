use crate::structures::clause::stored_clause::StoredClause;
use slotmap::{DefaultKey, SlotMap};

pub struct ClauseStore {
    pub formula_clauses: SlotMap<DefaultKey, StoredClause>,
    pub learnt_clauses: SlotMap<DefaultKey, StoredClause>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ClauseKey {
    Formula(slotmap::DefaultKey),
    Learnt(slotmap::DefaultKey),
}

pub fn retreive(store: &ClauseStore, key: ClauseKey) -> &StoredClause {
    match key {
        ClauseKey::Formula(key) => &store.formula_clauses[key],
        ClauseKey::Learnt(key) => &store.learnt_clauses[key],
    }
}
