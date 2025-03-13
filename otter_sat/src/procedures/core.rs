/*!
A procedure to obtain the unsatisfiable core of a(n unsatisfiable) clause.
 */
use std::collections::{HashSet, VecDeque};

use crate::{
    context::{ContextState, GenericContext},
    db::ClauseKey,
    structures::{atom::Atom, clause::Clause, consequence::AssignmentSource, literal::Literal},
};

impl<R: rand::Rng + std::default::Default> GenericContext<R> {
    /// Identifies the originals keys in the resolution graph of `key`.
    /// Note, in this context, resolution graphs are reflexive.
    pub fn original_keys(&self, key: ClauseKey) -> HashSet<ClauseKey> {
        let mut original_keys: HashSet<ClauseKey> = HashSet::default();
        let mut queue: VecDeque<ClauseKey> = VecDeque::default();

        queue.push_back(key);

        while let Some(key) = queue.pop_front() {
            match key {
                ClauseKey::OriginalUnit(_)
                | ClauseKey::OriginalBinary(_)
                | ClauseKey::Original(_) => {
                    original_keys.insert(key);
                }

                ClauseKey::AdditionUnit(_)
                | ClauseKey::AdditionBinary(_)
                | ClauseKey::Addition(_, _) => match self.clause_db.resolution_graph.get(&key) {
                    None => panic!("! Incomplete resolution graph"),

                    Some(keys) => {
                        for key in keys {
                            queue.push_back(*key);
                        }
                    }
                },
            }
        }

        original_keys
    }

    /// A collection of keys which identify an unsatisfiable core of a(n unsatisfiable) clause.
    ///
    /// The general technique is inspired by the source of MiniSAT.
    pub fn core_keys(&self) -> Vec<ClauseKey> {
        let ContextState::Unsatisfiable(unsat_key) = self.state else {
            todo!("Error path");
        };

        let mut seen_atoms: HashSet<Atom> = HashSet::default();
        let mut core: HashSet<ClauseKey> = HashSet::default();

        let mut todo: VecDeque<ClauseKey> = VecDeque::default();

        todo.push_back(unsat_key);
        for key in self.original_keys(unsat_key) {
            core.insert(key);
            for literal in unsafe { self.clause_db.get_unchecked(&key).literals() } {
                seen_atoms.insert(literal.atom());
            }
        }

        for literal in unsafe { self.clause_db.get_unchecked(&unsat_key) }.literals() {
            seen_atoms.insert(literal.atom());
        }

        for assignment in self.atom_db.assignments.iter().rev() {
            match assignment.source {
                AssignmentSource::PureLiteral => {}

                AssignmentSource::BCP(key) => {
                    for key in self.original_keys(key) {
                        core.insert(key);
                        for literal in unsafe { self.clause_db.get_unchecked(&key).literals() } {
                            seen_atoms.insert(literal.atom());
                        }
                    }
                }

                AssignmentSource::Decision => {}

                AssignmentSource::Assumption => {}

                AssignmentSource::Original => {
                    if seen_atoms.contains(&assignment.literal().atom()) {
                        core.insert(ClauseKey::OriginalUnit(*assignment.literal()));
                    }
                }

                AssignmentSource::Addition => {
                    let key = ClauseKey::OriginalUnit(*assignment.literal());

                    for key in self.original_keys(key) {
                        core.insert(key);
                        for literal in unsafe { self.clause_db.get_unchecked(&key).literals() } {
                            seen_atoms.insert(literal.atom());
                        }
                    }
                }
            }
        }

        core.into_iter().collect::<Vec<_>>()
    }
}
