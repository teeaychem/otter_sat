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

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ClauseKey {
    Formula(usize),
    Learned(usize),
}

impl ClauseKey {
    pub fn index(&self) -> usize {
        match self {
            Self::Formula(i) => *i,
            Self::Learned(i) => *i,
        }
    }
}

pub struct ClauseStore {
    keys: Vec<ClauseKey>,
    formula: Vec<StoredClause>,
    learned: Vec<Option<StoredClause>>,
}

#[allow(clippy::derivable_impls)]
impl Default for ClauseStore {
    fn default() -> Self {
        ClauseStore {
            keys: Vec::new(),
            formula: Vec::new(),
            learned: Vec::new(),
        }
    }
}

impl ClauseStore {
    fn new_formula_id(&self) -> ClauseKey {
        ClauseKey::Formula(self.formula.len())
    }

    fn new_learned_id(&self) -> ClauseKey {
        ClauseKey::Learned(self.learned.len())
    }

    pub fn with_capacity(capacity: usize) -> Self {
        ClauseStore {
            keys: Vec::new(),
            formula: Vec::with_capacity(capacity),
            learned: Vec::with_capacity(capacity),
        }
    }

    pub fn retreive_carefully(&self, key: ClauseKey) -> Option<&StoredClause> {
        match key {
            ClauseKey::Formula(key) => self.formula.get(key),
            ClauseKey::Learned(key) => match self.learned.get(key) {
                Some(Some(clause)) => Some(clause),
                _ => None,
            },
        }
    }

    pub fn retreive(&self, key: ClauseKey) -> &StoredClause {
        match key {
            ClauseKey::Formula(key) => unsafe { self.formula.get_unchecked(key) },
            ClauseKey::Learned(key) => unsafe {
                match self.learned.get_unchecked(key) {
                    Some(clause) => clause,
                    None => panic!("no"),
                }
            },
        }
    }

    pub fn retreive_carefully_mut(&mut self, key: ClauseKey) -> Option<&mut StoredClause> {
        match key {
            ClauseKey::Formula(key) => self.formula.get_mut(key),
            ClauseKey::Learned(key) => match self.learned.get_mut(key) {
                Some(Some(clause)) => Some(clause),
                _ => None,
            },
        }
    }

    pub fn retreive_mut(&mut self, key: ClauseKey) -> &mut StoredClause {
        match key {
            ClauseKey::Formula(key) => unsafe { self.formula.get_unchecked_mut(key) },
            ClauseKey::Learned(key) => unsafe {
                match self.learned.get_unchecked_mut(key) {
                    Some(clause) => clause,
                    None => panic!("no"),
                }
            },
        }
    }

    pub fn insert(
        &mut self,
        source: ClauseSource,
        clause: Vec<Literal>,
        variables: &impl VariableList,
    ) -> ClauseKey {
        // println!("{:?}", self.formula.iter().map(|c| c.key()).collect::<Vec<_>>());
        match source {
            ClauseSource::Formula => {
                let key = self.new_formula_id();
                self.formula
                    .push(StoredClause::new_from(key, clause, source, variables));
                key
            }
            ClauseSource::Resolution => {
                log::trace!("Learning clause {}", clause.as_string());
                match self.keys.len() {
                    0 => {
                        let key = self.new_learned_id();
                        self.learned
                            .push(Some(StoredClause::new_from(key, clause, source, variables)));
                        key
                    }
                    _ => {
                        let key = self.keys.pop().unwrap();
                        self.learned[key.index()] =
                            Some(StoredClause::new_from(key, clause, source, variables));
                        key
                    }
                }
            }
        }
    }

    pub fn formula_count(&self) -> usize {
        self.formula.len()
    }

    pub fn learned_count(&self) -> usize {
        self.learned.len()
    }

    pub fn formula_clauses(&self) -> impl Iterator<Item = impl Iterator<Item = Literal> + '_> + '_ {
        self.formula
            .iter()
            .map(|clause| clause.literal_slice().iter().copied())
    }

    // TODO: figure some improvement…
    pub fn reduce(&mut self, variables: &impl VariableList, glue_strength: config::GlueStrength) {
        let limit = self.learned_count() / 2;

        for index in 0..self.learned.len() {
            if let Some(clause) = unsafe { self.learned.get_unchecked(index) } {
                if self.keys.len() > limit {
                    break;
                } else if clause.lbd(variables) > glue_strength {
                    self.keys.push(clause.key());
                    unsafe { *self.learned.get_unchecked_mut(index) = None };
                }
            }
        }
        log::debug!(target: "forget", "Reduced to: {}", self.learned.len());
    }
}
