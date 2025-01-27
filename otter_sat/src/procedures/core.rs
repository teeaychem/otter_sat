use std::collections::{HashSet, VecDeque};

use crate::{
    context::{ContextState, GenericContext},
    db::ClauseKey,
    structures::{clause::Clause, literal::Literal},
};

impl<R: rand::Rng + std::default::Default> GenericContext<R> {
    pub fn core_keys(&self) -> Vec<ClauseKey> {
        let ContextState::Unsatisfiable(key) = self.state else {
            todo!("Error path");
        };

        let mut core: HashSet<ClauseKey> = HashSet::default();

        let mut seen: HashSet<ClauseKey> = HashSet::default();
        let mut todo: VecDeque<ClauseKey> = VecDeque::default();

        match key {
            ClauseKey::OriginalUnit(_) => panic!("!"),

            _ => todo.push_back(key),
        }

        let unsatisfiable_clause = self.clause_db.get(&key).expect("Final clause missing");

        for literal in unsatisfiable_clause.literals() {
            let negation = literal.negate();

            let literal_key = ClauseKey::AdditionUnit(negation);

            match self.clause_db.get(&literal_key) {
                Err(_) => {
                    core.insert(ClauseKey::OriginalUnit(negation));
                }
                Ok(_) => {
                    todo.push_back(ClauseKey::AdditionUnit(negation));
                }
            }
        }

        while let Some(key) = todo.pop_front() {
            if !seen.insert(key) {
                continue;
            }

            match key {
                ClauseKey::OriginalUnit(_) => {
                    core.insert(key);
                }

                ClauseKey::AdditionUnit(unit) => {
                    let db_clause = unsafe {
                        self.clause_db
                            .get_unchecked(&key)
                            .expect("Missing core clause")
                    };

                    match db_clause.premises().len() {
                        0 => panic!("!"),

                        1 => {
                            let the_premise_key = db_clause.premises().iter().next().unwrap();
                            let the_premise =
                                unsafe { self.clause_db.get_unchecked(the_premise_key) }.unwrap();

                            for key in the_premise.premises() {
                                todo.push_back(*key);
                            }

                            for literal in the_premise.literals() {
                                if literal == &unit {
                                    continue;
                                }

                                let literal_key = ClauseKey::AdditionUnit(literal.negate());

                                match self.clause_db.get(&literal_key) {
                                    Err(_) => {
                                        core.insert(ClauseKey::OriginalUnit(literal.negate()));
                                    }
                                    Ok(_) => {
                                        todo.push_back(literal_key);
                                    }
                                }
                            }
                        }

                        _ => {
                            for key in db_clause.premises() {
                                todo.push_back(*key);
                            }
                        }
                    }
                }

                ClauseKey::Binary(_) => {
                    core.insert(key);
                    let clause = unsafe {
                        self.clause_db
                            .get_unchecked(&key)
                            .expect("Missing core clause")
                    };
                    for key in clause.premises() {
                        todo.push_back(*key);
                    }
                }

                ClauseKey::Original(_) => {
                    core.insert(key);
                }

                ClauseKey::Addition(_, _) => {
                    let clause = unsafe {
                        self.clause_db
                            .get_unchecked(&key)
                            .expect("Missing core clause")
                    };
                    for key in clause.premises() {
                        todo.push_back(*key);
                    }
                }
            }
        }

        core.iter().cloned().collect::<Vec<_>>()
    }
}
