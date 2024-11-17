use std::{
    collections::{BTreeSet, HashMap, VecDeque},
    sync::{Arc, Mutex},
};

use otter_lib::{
    db::keys::ClauseKey,
    dispatch::{
        delta::{self},
        Dispatch,
    },
    structures::{
        clause::Clause,
        literal::{Literal, LiteralT},
    },
};

/* A placeholder awaiting interest…

  To recover a core implication history is needed.
  - For proven literals, this is recorded for BCP.
  - For clauses, this is obtained by inspecting the resolution trail of the clause.
    Original clauses, of course, are part of the core.

 Recovering the core is then a matter of tracing the derivation of literals in the unsatisfiable assignment through used clauses.

  The core database buffers BCP and resolution information, and then associates this with a subsequently received key in an appropriate (hash) map.
  This could be made a little more elegant, but this is simple.

 A minor issue is that subsumption may `corrupt` an original formula, so a map from formula keys to their initial form is also kept.
*/
#[derive(Default, Debug)]
pub struct CoreDB {
    pub conflict: Option<ClauseKey>,
    original_map: HashMap<ClauseKey, Vec<Literal>>,
    resolution_buffer: Vec<ClauseKey>,
    resolution_q: VecDeque<Vec<ClauseKey>>,
    bcp_buffer: Option<(Literal, ClauseKey, Literal)>,
    pub clause_map: HashMap<ClauseKey, Vec<ClauseKey>>,
    pub literal_map: HashMap<Literal, Vec<ClauseKey>>,
}

impl CoreDB {
    pub fn core_clauses(&self) -> Result<Vec<Vec<Literal>>, ()> {
        let mut core_q = std::collections::VecDeque::<ClauseKey>::new();
        let mut key_set = std::collections::BTreeSet::new();
        let mut literal_set: BTreeSet<Literal> = std::collections::BTreeSet::new();
        let mut core_raw = std::collections::BTreeSet::new();

        let Some(conflict_key) = self.conflict else {
            return Err(());
        };

        // start with the conflict, then loop
        core_q.push_back(conflict_key);

        /*
        key set ensures processing only happens on a fresh key

        if the key is for a formula, then clause is recorded and the literals of the clause are checked against the observed literals
        otherwise, the clauses used when resolving the learnt clause are added

         when checking literals, if the negation of the literal has been observed at level 0 then it was relevant to the conflict
         so, if the literal was obtained either by resolution or directly from some clause, then that clause or the clauses used for resolution are added to the q
         this skips assumed literals
         */
        while let Some(key) = core_q.pop_front() {
            if key_set.insert(key) {
                match key {
                    ClauseKey::Formula(_) => {
                        let the_clause = self.original_map.get(&key).unwrap();
                        core_raw.insert(the_clause.clone());
                        for literal in the_clause.literals() {
                            if literal_set.insert(*literal) {
                                if let Some(past) = self.literal_map.get(&literal.negate()) {
                                    for source_key in past {
                                        core_q.push_back(*source_key)
                                    }
                                }
                            }
                        }
                    }

                    ClauseKey::Binary(_) => match self.clause_map.get(&key) {
                        None => {
                            let the_clause = self.original_map.get(&key).unwrap();
                            core_raw.insert(the_clause.clone());
                            for literal in the_clause.literals() {
                                if literal_set.insert(*literal) {
                                    if let Some(past) = self.literal_map.get(&literal.negate()) {
                                        for source_key in past {
                                            core_q.push_back(*source_key)
                                        }
                                    }
                                }
                            }
                        }
                        Some(keys) => {
                            for source_key in keys {
                                core_q.push_back(*source_key);
                            }
                        }
                    },

                    ClauseKey::Learned(_, _) => match self.clause_map.get(&key) {
                        None => {
                            panic!("missed {key}")
                        }
                        Some(keys) => {
                            for source_key in keys {
                                core_q.push_back(*source_key);
                            }
                        }
                    },
                }
            }
        }

        Ok(core_raw.into_iter().collect())
    }
}

/* This is fairly gnarly.
  The goal is to acquire a single lock on a core database for the duration of the returned function.
  This, while also allowing for the core database to be optional within the context of whatever calls this.
  Simplest, then, seems to be passing in the core database as it would be found in the context…
*/
#[allow(clippy::single_match)]
#[allow(clippy::collapsible_match)]
pub fn core_db_builder<'g>(
    core_db_ptr: &'g Option<Arc<Mutex<CoreDB>>>,
) -> Box<dyn FnMut(&Dispatch) + 'g> {
    let mut the_core_db = core_db_ptr.as_ref().unwrap().lock().unwrap();
    let handler = move |dispatch: &Dispatch| {
        match dispatch {
            Dispatch::Resolution(delta) => {
                //
                match delta {
                    delta::Resolution::Begin => {
                        //
                        assert!(the_core_db.resolution_buffer.is_empty())
                    }
                    delta::Resolution::End => {
                        let the_clause = std::mem::take(&mut the_core_db.resolution_buffer);
                        the_core_db.resolution_q.push_back(the_clause)
                    }
                    delta::Resolution::Used(k) => the_core_db.resolution_buffer.push(*k),
                    delta::Resolution::Subsumed(_, _) => {
                        // TODO: maybe
                    }
                }
            }
            Dispatch::ClauseDB(delta) => match delta {
                delta::ClauseDB::BinaryResolution(key, _)
                | delta::ClauseDB::TransferBinary(_, key, _)
                | delta::ClauseDB::Learned(key, _) => {
                    let the_sources = the_core_db.resolution_q.pop_front();
                    the_core_db
                        .clause_map
                        .insert(*key, the_sources.expect("q miss"));
                }
                delta::ClauseDB::Original(key, clause) => {
                    the_core_db.original_map.insert(*key, clause.clone());
                }
                delta::ClauseDB::BinaryOriginal(key, clause) => {
                    the_core_db.original_map.insert(*key, clause.clone());
                }
                delta::ClauseDB::Deletion(_, _) => {}
            },
            Dispatch::Level(delta) => {
                //
                match delta {
                    delta::Level::Assumption(_) | delta::Level::Pure(_) => {}
                    delta::Level::ResolutionProof(literal) => {
                        let the_sources = the_core_db.resolution_q.pop_front();
                        the_core_db
                            .literal_map
                            .insert(*literal, the_sources.expect("q miss"));
                    }
                    delta::Level::Proof(_) => {
                        let Some((_, clause, to)) = the_core_db.bcp_buffer.take() else {
                            panic!("empty bcp buffer");
                        };
                        the_core_db.literal_map.insert(to, vec![clause]);
                    }
                    delta::Level::Forced(key, literal) => {
                        the_core_db.literal_map.insert(*literal, vec![*key]);
                    }
                }
            }
            Dispatch::VariableDB(delta) => {
                //
                match delta {
                    delta::Variable::Unsatisfiable(key) => the_core_db.conflict = Some(*key),
                    _ => {}
                }
            }
            Dispatch::BCP(delta) => {
                //
                match delta {
                    delta::BCP::Instance(from, using, to) => {
                        the_core_db.bcp_buffer = Some((*from, *using, *to));
                    }
                    delta::BCP::Conflict(_, _) => {}
                }
            }
            _ => {}
        }
    };
    Box::new(handler)
}
