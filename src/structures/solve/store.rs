use crate::structures::{
    clause::{
        stored::{Source as ClauseSource, StoredClause},
        Clause,
    },
    literal::Literal,
    solve::config,
    valuation::Valuation,
    variable::Variable,
};

use slotmap::{DefaultKey, SlotMap};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ClauseKey {
    Formula(slotmap::DefaultKey),
    Learnt(slotmap::DefaultKey),
}

pub struct ClauseStore {
    formula: SlotMap<DefaultKey, StoredClause>,
    learned: SlotMap<DefaultKey, StoredClause>,
}

impl ClauseStore {
    pub fn new() -> Self {
        ClauseStore {
            formula: SlotMap::new(),
            learned: SlotMap::new(),
        }
    }

    pub fn retreive<'a>(&'a self, key: ClauseKey) -> Option<&'a StoredClause> {
        match key {
            ClauseKey::Formula(key) => self.formula.get(key),
            ClauseKey::Learnt(key) => self.learned.get(key),
        }
    }

    pub fn retreive_unsafe<'a>(&'a self, key: ClauseKey) -> &'a StoredClause {
        match key {
            ClauseKey::Formula(key) => unsafe { self.formula.get_unchecked(key) },
            ClauseKey::Learnt(key) => unsafe { self.learned.get_unchecked(key) },
        }
    }

    pub fn retreive_mut<'a>(&'a mut self, key: ClauseKey) -> Option<&'a mut StoredClause> {
        match key {
            ClauseKey::Formula(key) => self.formula.get_mut(key),
            ClauseKey::Learnt(key) => self.learned.get_mut(key),
        }
    }

    pub fn insert(
        &mut self,
        source: ClauseSource,
        clause: Vec<Literal>,
        valuation: &impl Valuation,
        variables: &mut [Variable],
    ) -> ClauseKey {
        match source {
            ClauseSource::Formula => {
                let key = self.formula.insert_with_key(|k| {
                    StoredClause::new_from(
                        ClauseKey::Formula(k),
                        clause,
                        source,
                        valuation,
                        variables,
                    )
                });

                ClauseKey::Formula(key)
            }
            ClauseSource::Resolution(_) => {
                log::trace!("Learning clause {}", clause.as_string());

                let key = self.learned.insert_with_key(|k| {
                    let clause = StoredClause::new_from(
                        ClauseKey::Learnt(k),
                        clause,
                        source,
                        valuation,
                        variables,
                    );
                    clause.set_lbd(variables);
                    clause
                });

                ClauseKey::Learnt(key)
            }
        }
    }

    pub fn learned_count(&self) -> usize {
        self.learned.len()
    }

    pub fn clauses(&self) -> impl Iterator<Item = impl Iterator<Item = Literal> + '_> + '_ {
        self.formula
            .iter()
            .chain(&self.learned)
            .map(|(_, clause)| clause.literal_slice().iter().copied())
    }

    // TODO: figure some improvement…
    pub fn reduce(&mut self) {
        let limit = self.learned_count();
        let mut keys_to_drop = vec![];
        for (k, v) in &self.learned {
            if keys_to_drop.len() > limit {
                break;
            } else if v.get_set_lbd() > unsafe { config::GLUE_STRENGTH } {
                keys_to_drop.push(k);
            }
        }

        for key in keys_to_drop {
            self.learned.remove(key);
        }
        log::debug!(target: "forget", "Reduced to: {}", self.learned.len());
    }
}
