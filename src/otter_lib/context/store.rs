use crate::{
    config::{self},
    structures::{
        clause::{
            stored::{ClauseSource, StoredClause},
            Clause,
        },
        literal::Literal,
        variable::list::VariableList,
    },
};

type FormulaIndex = u32;
type FormulaReuse = u16;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ClauseKey {
    Formula(FormulaIndex),
    Learned(FormulaIndex, FormulaReuse),
}

impl ClauseKey {
    pub fn index(&self) -> usize {
        match self {
            Self::Formula(i) => *i as usize,
            Self::Learned(i, _) => *i as usize,
        }
    }

    pub fn usage(&self) -> FormulaReuse {
        match self {
            Self::Formula(_) => panic!("Can't `use` formula keys"),
            Self::Learned(_, usage) => *usage,
        }
    }

    pub fn reuse(&self) -> Self {
        match self {
            Self::Formula(_) => panic!("Can't reuse formula keys"),
            Self::Learned(index, reuse) => {
                assert!(*reuse < FormulaReuse::MAX);
                ClauseKey::Learned(*index, reuse + 1)
            }
        }
    }
}

pub struct ClauseStore {
    keys: Vec<ClauseKey>,
    formula: Vec<StoredClause>,
    formula_count: FormulaIndex,
    learned: Vec<Option<StoredClause>>,
    pub learned_count: FormulaIndex,
    pub resolution_graph: Vec<Vec<Vec<ClauseKey>>>,
}

#[allow(clippy::derivable_impls)]
impl Default for ClauseStore {
    fn default() -> Self {
        ClauseStore {
            keys: Vec::new(),
            formula: Vec::new(),
            formula_count: 0,
            learned: Vec::new(),
            learned_count: 0,
            resolution_graph: Vec::new(),
        }
    }
}

impl ClauseStore {
    fn new_formula_id(&mut self) -> ClauseKey {
        assert!(self.formula_count < FormulaIndex::MAX);
        let key = ClauseKey::Formula(self.formula_count);
        self.formula_count += 1;
        key
    }

    fn new_learned_id(&mut self) -> ClauseKey {
        assert!(self.learned_count < FormulaIndex::MAX);
        let key = ClauseKey::Learned(self.learned_count, 0);
        self.learned_count += 1;
        key
    }

    pub fn with_capacity(capacity: usize) -> Self {
        ClauseStore {
            keys: Vec::new(),
            formula: Vec::with_capacity(capacity),
            formula_count: 0,
            learned: Vec::with_capacity(capacity),
            learned_count: 0,
            resolution_graph: Vec::with_capacity(capacity),
        }
    }

    pub fn get_carefully(&self, key: ClauseKey) -> Option<&StoredClause> {
        match key {
            ClauseKey::Formula(index) => self.formula.get(index as usize),
            ClauseKey::Learned(index, reuse) => match self.learned.get(index as usize) {
                Some(Some(clause)) if clause.key().usage() == reuse => Some(clause),
                _ => None,
            },
        }
    }

    pub fn get(&self, key: ClauseKey) -> &StoredClause {
        match key {
            ClauseKey::Formula(index) => unsafe { self.formula.get_unchecked(index as usize) },
            ClauseKey::Learned(index, reuse) => unsafe {
                match self.learned.get_unchecked(index as usize) {
                    Some(clause) if clause.key().usage() == reuse => clause,
                    _ => panic!("no"),
                }
            },
        }
    }

    pub fn get_carefully_mut(&mut self, key: ClauseKey) -> Option<&mut StoredClause> {
        match key {
            ClauseKey::Formula(index) => self.formula.get_mut(index as usize),
            ClauseKey::Learned(index, reuse) => match self.learned.get_mut(index as usize) {
                Some(Some(clause)) if clause.key().usage() == reuse => Some(clause),
                _ => None,
            },
        }
    }

    pub fn get_mut(&mut self, key: ClauseKey) -> &mut StoredClause {
        match key {
            ClauseKey::Formula(index) => unsafe { self.formula.get_unchecked_mut(index as usize) },
            ClauseKey::Learned(index, reuse) => unsafe {
                match self.learned.get_unchecked_mut(index as usize) {
                    Some(clause) if clause.key().usage() == reuse => clause,
                    _ => panic!("no"),
                }
            },
        }
    }

    pub fn insert(
        &mut self,
        source: ClauseSource,
        clause: Vec<Literal>,
        variables: &impl VariableList,
        resolution_keys: Option<Vec<ClauseKey>>,
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
                        self.resolution_graph.push(vec![
                            resolution_keys.expect("missing resolution info for learnt")
                        ]);
                        assert_eq!(self.resolution_graph[key.index()].len(), 1);
                        key
                    }
                    _ => unsafe {
                        let key = self.keys.pop().unwrap().reuse();
                        *self.learned.get_unchecked_mut(key.index()) =
                            Some(StoredClause::new_from(key, clause, source, variables));
                        self.resolution_graph[key.index()]
                            .push(resolution_keys.expect("missing resolution info for learnt"));
                        assert_eq!(
                            self.resolution_graph[key.index()].len(),
                            key.usage() as usize + 1
                        );
                        key
                    },
                }
            }
        }
    }

    pub fn formula_count(&self) -> usize {
        self.formula_count as usize
    }

    pub fn learned_count(&self) -> usize {
        self.learned_count as usize
    }

    pub fn formula_clauses(&self) -> impl Iterator<Item = impl Iterator<Item = Literal> + '_> + '_ {
        self.formula
            .iter()
            .map(|clause| clause.literal_slice().iter().copied())
    }

    // TODO: figure some improvement…
    pub fn reduce(&mut self, variables: &impl VariableList, glue_strength: config::GlueStrength) {
        let limit = self.learned_count as usize / 2;

        for index in 0..self.learned_count {
            if let Some(clause) = unsafe { self.learned.get_unchecked(index as usize) } {
                if self.keys.len() > limit {
                    break;
                } else if clause.lbd(variables) > glue_strength {
                    self.keys.push(clause.key());
                    unsafe { *self.learned.get_unchecked_mut(index as usize) = None };
                }
            }
        }
        log::debug!(target: "forget", "Reduced to: {}", self.learned_count);
    }
}
