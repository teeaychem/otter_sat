use crate::{
    config::{self},
    structures::{
        clause::{
            stored::{Source as ClauseSource, StoredClause},
            Clause,
        },
        literal::Literal,
        variable::list::VariableList,
    },
};

use slotmap::{DefaultKey, SlotMap};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ClauseKey {
    Formula(slotmap::DefaultKey),
    Learned(slotmap::DefaultKey),
}

pub struct ClauseStore {
    formula: SlotMap<DefaultKey, StoredClause>,
    learned: SlotMap<DefaultKey, StoredClause>,
}

impl Default for ClauseStore {
    fn default() -> Self {
        ClauseStore {
            formula: SlotMap::new(),
            learned: SlotMap::new(),
        }
    }
}

impl ClauseStore {
    pub fn with_capacity(capacity: usize) -> Self {
        ClauseStore {
            formula: SlotMap::with_capacity(capacity),
            learned: SlotMap::with_capacity(capacity),
        }
    }

    pub fn retreive_carefully(&self, key: ClauseKey) -> Option<&StoredClause> {
        match key {
            ClauseKey::Formula(key) => self.formula.get(key),
            ClauseKey::Learned(key) => self.learned.get(key),
        }
    }

    pub fn retreive(&self, key: ClauseKey) -> &StoredClause {
        match key {
            ClauseKey::Formula(key) => unsafe { self.formula.get_unchecked(key) },
            ClauseKey::Learned(key) => unsafe { self.learned.get_unchecked(key) },
        }
    }

    pub fn retreive_carefully_mut(&mut self, key: ClauseKey) -> Option<&mut StoredClause> {
        match key {
            ClauseKey::Formula(key) => self.formula.get_mut(key),
            ClauseKey::Learned(key) => self.learned.get_mut(key),
        }
    }

    pub fn retreive_mut(&mut self, key: ClauseKey) -> &mut StoredClause {
        match key {
            ClauseKey::Formula(key) => unsafe { self.formula.get_unchecked_mut(key) },
            ClauseKey::Learned(key) => unsafe { self.learned.get_unchecked_mut(key) },
        }
    }

    pub fn insert(
        &mut self,
        source: ClauseSource,
        clause: Vec<Literal>,
        variables: &impl VariableList,
    ) -> ClauseKey {
        match source {
            ClauseSource::Formula => {
                let key = self.formula.insert_with_key(|k| {
                    StoredClause::new_from(ClauseKey::Formula(k), clause, source, variables)
                });
                ClauseKey::Formula(key)
            }
            ClauseSource::Resolution => {
                log::trace!("Learning clause {}", clause.as_string());

                let key = self.learned.insert_with_key(|k| {
                    StoredClause::new_from(ClauseKey::Learned(k), clause, source, variables)
                });
                ClauseKey::Learned(key)
            }
        }
    }

    pub fn formula_count(&self) -> usize {
        self.formula.len()
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
    pub fn reduce(&mut self, variables: &impl VariableList, glue_strength: config::GlueStrength) {
        let limit = self.learned_count() / 2;
        let mut keys_to_drop = vec![];

        for (k, v) in &self.learned {
            if keys_to_drop.len() > limit {
                break;
            } else if v.lbd(variables) > glue_strength {
                keys_to_drop.push(k);
            }
        }

        for key in keys_to_drop {
            self.learned.remove(key);
        }
        log::debug!(target: "forget", "Reduced to: {}", self.learned.len());
    }
}