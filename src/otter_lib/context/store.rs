use std::fmt::Display;

use crate::{
    config::{self, ClauseActivity, Config, GlueStrength},
    generic::heap::IndexHeap,
    structures::{
        clause::{
            stored::{ClauseSource, StoredClause},
            Clause,
        },
        literal::Literal,
        variable::{delegate::VariableStore, WatchElement},
    },
};

type FormulaIndex = u32;
type FormulaReuse = u16;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ClauseKey {
    Formula(FormulaIndex),
    Binary(FormulaIndex),
    Learned(FormulaIndex, FormulaReuse),
}

impl Display for ClauseKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Formula(key) => write!(f, "Formula({key})"),
            Self::Learned(key, usage) => write!(f, "Learned({key}, {usage})"),
            Self::Binary(key) => write!(f, "Learned({key})"),
        }
    }
}

impl ClauseKey {
    pub fn index(&self) -> usize {
        match self {
            Self::Formula(i) => *i as usize,
            Self::Binary(i) => *i as usize,
            Self::Learned(i, _) => *i as usize,
        }
    }

    pub fn usage(&self) -> FormulaReuse {
        match self {
            Self::Formula(_) => panic!("can't `use` formula keys"),
            Self::Binary(_) => panic!("can't `use` binary keys"),
            Self::Learned(_, usage) => *usage,
        }
    }

    pub fn reuse(&self) -> Self {
        match self {
            Self::Formula(_) => panic!("Can't reuse formula keys"),
            Self::Binary(_) => panic!("Can't reuse binary keys"),
            Self::Learned(index, reuse) => {
                assert!(*reuse < FormulaReuse::MAX);
                ClauseKey::Learned(*index, reuse + 1)
            }
        }
    }
}

pub struct ActivityGlue {
    pub activity: ClauseActivity,
    pub lbd: GlueStrength,
}

impl Default for ActivityGlue {
    fn default() -> Self {
        ActivityGlue {
            activity: 0.0,
            lbd: 0,
        }
    }
}

// `Revered` as max heap
impl PartialOrd for ActivityGlue {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        let lbd_comparison = match self.lbd.cmp(&other.lbd) {
            std::cmp::Ordering::Less => std::cmp::Ordering::Greater,
            std::cmp::Ordering::Greater => std::cmp::Ordering::Less,
            std::cmp::Ordering::Equal => match self.activity.partial_cmp(&other.activity) {
                None => panic!("could not compare activity/lbd"),
                Some(comparison) => match comparison {
                    std::cmp::Ordering::Less => std::cmp::Ordering::Greater,
                    std::cmp::Ordering::Greater => std::cmp::Ordering::Less,
                    std::cmp::Ordering::Equal => std::cmp::Ordering::Equal,
                },
            },
        };
        Some(lbd_comparison)
    }
}

// impl PartialOrd for ActivityGlue {
//     fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
//         let lbd_comparison = match self.activity.partial_cmp(&other.activity) {
//             None => panic!("could not compare activity/lbd"),
//             Some(comparison) => match comparison {
//                 std::cmp::Ordering::Less => std::cmp::Ordering::Greater,
//                 std::cmp::Ordering::Greater => std::cmp::Ordering::Less,
//                 std::cmp::Ordering::Equal => match self.lbd.cmp(&other.lbd) {
//                     std::cmp::Ordering::Less => std::cmp::Ordering::Greater,
//                     std::cmp::Ordering::Greater => std::cmp::Ordering::Less,
//                     std::cmp::Ordering::Equal => std::cmp::Ordering::Equal,
//                 },
//             },
//         };
//         Some(lbd_comparison)
//     }
// }

impl PartialEq for ActivityGlue {
    fn eq(&self, other: &Self) -> bool {
        self.lbd.eq(&other.lbd) && self.activity.eq(&other.activity)
    }
}

pub struct ClauseStore {
    counts: ClauseStoreCounts,
    keys: Vec<ClauseKey>,
    formula: Vec<StoredClause>,

    pub binary_graph: Vec<Vec<ClauseKey>>,
    learned: Vec<Option<StoredClause>>,

    pub learned_slots: FormulaIndex,

    pub resolution_graph: Vec<Vec<Vec<ClauseKey>>>,
    binary: Vec<StoredClause>,

    pub learned_activity: IndexHeap<ActivityGlue>,
    pub learned_increment: ClauseActivity,
}

pub struct ClauseStoreCounts {
    formula: FormulaIndex,
    binary: FormulaIndex,
    learned: FormulaIndex,
}

#[allow(clippy::derivable_impls)]
impl Default for ClauseStoreCounts {
    fn default() -> Self {
        ClauseStoreCounts {
            formula: 0,
            binary: 0,
            learned: 0,
        }
    }
}

#[allow(clippy::derivable_impls)]
impl Default for ClauseStore {
    fn default() -> Self {
        ClauseStore {
            counts: ClauseStoreCounts::default(),
            keys: Vec::new(),
            formula: Vec::new(),
            learned: Vec::new(),
            learned_slots: 0,
            binary: Vec::new(),
            binary_graph: Vec::new(),
            resolution_graph: Vec::new(),
            learned_activity: IndexHeap::default(),
            learned_increment: ClauseActivity::default(),
        }
    }
}

impl ClauseStore {
    fn new_formula_id(&mut self) -> ClauseKey {
        assert!(self.counts.formula < FormulaIndex::MAX);
        let key = ClauseKey::Formula(self.counts.formula);
        self.counts.formula += 1;
        key
    }

    fn new_binary_id(&mut self) -> ClauseKey {
        assert!(self.counts.binary < FormulaIndex::MAX);
        let key = ClauseKey::Binary(self.counts.binary);
        self.counts.binary += 1;
        key
    }

    fn new_learned_id(&mut self) -> ClauseKey {
        assert!(self.learned_slots < FormulaIndex::MAX);
        let key = ClauseKey::Learned(self.learned_slots, 0);
        self.learned_slots += 1;
        key
    }

    pub fn with_capacity(capacity: usize) -> Self {
        ClauseStore {
            counts: ClauseStoreCounts::default(),
            keys: Vec::new(),
            formula: Vec::with_capacity(capacity),
            learned: Vec::with_capacity(capacity),
            learned_slots: 0,
            binary: Vec::new(),
            binary_graph: Vec::with_capacity(capacity),
            resolution_graph: Vec::with_capacity(capacity),
            learned_activity: IndexHeap::new(capacity),
            learned_increment: ClauseActivity::default(),
        }
    }

    pub fn get_carefully(&self, key: ClauseKey) -> Option<&StoredClause> {
        match key {
            ClauseKey::Formula(index) => self.formula.get(index as usize),
            ClauseKey::Binary(index) => self.binary.get(index as usize),
            ClauseKey::Learned(index, reuse) => match self.learned.get(index as usize) {
                Some(Some(clause)) if clause.key().usage() == reuse => Some(clause),
                _ => None,
            },
        }
    }

    pub fn get(&self, key: ClauseKey) -> &StoredClause {
        match key {
            ClauseKey::Formula(index) => unsafe { self.formula.get_unchecked(index as usize) },
            ClauseKey::Binary(index) => unsafe { self.binary.get_unchecked(index as usize) },
            ClauseKey::Learned(index, reuse) => unsafe {
                match self.learned.get_unchecked(index as usize) {
                    Some(clause) if clause.key().usage() == reuse => clause,
                    None => panic!("missing {key:?}"),
                    Some(_) => panic!("reuse {key:?}"),
                }
            },
        }
    }

    pub fn get_carefully_mut(&mut self, key: ClauseKey) -> Option<&mut StoredClause> {
        match key {
            ClauseKey::Formula(index) => self.formula.get_mut(index as usize),
            ClauseKey::Binary(index) => self.binary.get_mut(index as usize),
            ClauseKey::Learned(index, reuse) => match self.learned.get_mut(index as usize) {
                Some(Some(clause)) if clause.key().usage() == reuse => Some(clause),
                _ => None,
            },
        }
    }

    pub fn get_mut(&mut self, key: ClauseKey) -> &mut StoredClause {
        match key {
            ClauseKey::Formula(index) => unsafe { self.formula.get_unchecked_mut(index as usize) },
            ClauseKey::Binary(index) => unsafe { self.binary.get_unchecked_mut(index as usize) },
            ClauseKey::Learned(index, reuse) => unsafe {
                match self.learned.get_unchecked_mut(index as usize) {
                    Some(clause) if clause.key().usage() == reuse => clause,
                    None => panic!("missing {key:?}"),
                    Some(_) => panic!("reuse {key:?}"),
                }
            },
        }
    }

    pub fn insert(
        &mut self,
        source: ClauseSource,
        clause: Vec<Literal>,
        subsumed: Vec<Literal>,
        variables: &mut VariableStore,
        resolution_keys: Option<Vec<ClauseKey>>,
    ) -> ClauseKey {
        match clause.len() {
            2 => {
                let key = self.new_binary_id();
                self.binary.push(StoredClause::new_from(
                    key, clause, subsumed, source, variables,
                ));
                self.binary_graph.push(resolution_keys.unwrap_or_default());
                key
            }
            _ => match source {
                ClauseSource::Formula => {
                    let key = self.new_formula_id();
                    self.formula.push(StoredClause::new_from(
                        key, clause, subsumed, source, variables,
                    ));
                    key
                }
                ClauseSource::Resolution => {
                    log::trace!(target: crate::log::targets::CLAUSE_STORE, "Learning clause {}", clause.as_string());
                    self.counts.learned += 1;
                    match self.keys.len() {
                        0 => {
                            let key = self.new_learned_id();
                            let the_clause =
                                StoredClause::new_from(key, clause, subsumed, source, variables);

                            let value = ActivityGlue {
                                activity: ClauseActivity::default(),
                                lbd: the_clause.lbd(variables),
                            };

                            self.learned.push(Some(the_clause));

                            self.learned_activity.insert(key.index(), value);
                            self.resolution_graph.push(vec![
                                resolution_keys.expect("missing resolution info for learnt")
                            ]);

                            assert_eq!(self.resolution_graph[key.index()].len(), 1);
                            key
                        }
                        _ => unsafe {
                            let key = self.keys.pop().unwrap().reuse();
                            let the_clause =
                                StoredClause::new_from(key, clause, subsumed, source, variables);

                            let value = ActivityGlue {
                                activity: ClauseActivity::default(),
                                lbd: the_clause.lbd(variables),
                            };

                            *self.learned.get_unchecked_mut(key.index()) = Some(the_clause);
                            self.learned_activity.insert(key.index(), value);
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
            },
        }
    }

    pub fn clause_count(&self) -> usize {
        (self.counts.formula + self.counts.learned + self.counts.binary) as usize
    }

    pub fn formula_clauses(&self) -> impl Iterator<Item = impl Iterator<Item = Literal> + '_> + '_ {
        self.formula
            .iter()
            .map(|clause| clause.literal_slice().iter().copied())
            .chain(
                self.binary
                    .iter()
                    .map(|clause| clause.literal_slice().iter().copied()),
            )
    }

    /*
    Removing from learned checks to ensure removal is ok
    As the elements are optional for reuse, take places None at the index, as would be needed anyway
     */
    pub fn remove_from_learned(&mut self, index: usize) -> StoredClause {
        if unsafe { self.learned.get_unchecked(index) }.is_none() {
            panic!("attempt to remove something that is not there")
        } else {
            // assert!(matches!(the_clause.key(), ClauseKey::LearnedLong(_, _)));
            let the_clause =
                std::mem::take(unsafe { self.learned.get_unchecked_mut(index) }).unwrap();
            self.learned_activity.remove(index);
            self.keys.push(the_clause.key());
            self.counts.learned -= 1;
            the_clause
        }
    }

    /*
    Cases:
    - Formula
      A new binary clause is created, and all occurrences of the formula clause in watch lists are removed

    - Learned
      Removes a long clause and then:
      - Replaces the key with a new binary key
      - Updates notifies literals of the new watch
      - Adds the clause to the binary vec
        This order is mostly dictated by the borrow checker
    */
    pub fn transfer_to_binary(
        &mut self,
        key: ClauseKey,
        variables: &mut VariableStore,
        literal: Literal,
    ) -> ClauseKey {
        match key {
            ClauseKey::Binary(_) => panic!("cannot transfer binary"),
            ClauseKey::Formula(index) => {
                let formula_clause = &self.formula[index as usize];
                let copied_clause = formula_clause.literal_slice().to_vec();
                let binary_key = self.new_binary_id();

                assert_eq!(copied_clause.len(), 2);

                variables.remove_watch(literal, key);
                variables.remove_watch(unsafe { *copied_clause.get_unchecked(0) }, key);
                variables.remove_watch(unsafe { *copied_clause.get_unchecked(1) }, key);

                // as a new clause is created there's no need to add watches as in the learnt case

                let binary_clause = StoredClause::new_from(
                    binary_key,
                    copied_clause,
                    Vec::default(),
                    ClauseSource::Resolution,
                    variables,
                );

                self.binary.push(binary_clause);
                self.binary_graph.push(vec![key]);
                binary_key
            }
            ClauseKey::Learned(_, _) => {
                let mut the_clause = self.remove_from_learned(key.index());

                let binary_key = self.new_binary_id();
                the_clause.key = binary_key;

                variables.remove_watch(literal, key);

                let a = unsafe { *the_clause.get_unchecked(0) };
                let b = unsafe { *the_clause.get_unchecked(1) };

                variables.remove_watch(a, key);
                variables.remove_watch(b, key);
                variables.add_watch(a, WatchElement::Binary(b, binary_key));
                variables.add_watch(b, WatchElement::Binary(a, binary_key));

                self.binary.push(the_clause);
                self.binary_graph.push(vec![key]);
                self.counts.learned += 1; // removing decrements the count

                binary_key
            }
        }
    }

    // TODO: figure some improvement…
    pub fn reduce(&mut self) {
        let limit = self.counts.learned as usize / 2;
        for _ in 0..limit {
            if let Some(index) = self.learned_activity.pop_max() {
                self.remove_from_learned(index);
            } else {
                panic!("reduce issue")
            }
        }
        log::debug!(target: crate::log::targets::REDUCTION, "Learnt clauses reduced to: {}", self.counts.learned);
    }

    pub fn bump_activity(&mut self, key: ClauseKey, config: &Config) {
        let bump_activity = |s: &ActivityGlue| ActivityGlue {
            activity: s.activity + config::defaults::CLAUSE_BUMP,
            lbd: s.lbd,
        };

        let activity = self.learned_activity.value_at(key.index()).activity;
        if activity + self.learned_increment > ClauseActivity::MAX {
            let factor = 1.0 / activity;
            let decay_activity = |s: &ActivityGlue| ActivityGlue {
                activity: s.activity * factor,
                lbd: s.lbd,
            };
            self.learned_activity.apply_to_all(decay_activity);
            self.learned_increment *= factor
        }

        self.learned_activity
            .apply_to_index(key.index(), bump_activity);

        let decay = config.clause_decay * 1e-3;
        let factor = 1.0 / (1.0 - decay);
        self.learned_increment *= factor
    }
}
