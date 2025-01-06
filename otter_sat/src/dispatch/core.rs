//! Tools for identifying unsatisfiable cores by using dispatches.
//!
//!

use std::{
    collections::{HashMap, VecDeque},
    sync::{Arc, Mutex},
};

use crate::{
    db::ClauseKey,
    dispatch::{
        library::delta::{self, AtomDB, ClauseDB, Delta, LiteralDB, Resolution, BCP},
        Dispatch,
    },
    structures::{
        clause::{vClause, Clause},
        literal::{abLiteral, Literal},
    },
    types::err::{self},
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

/// A database of information useful for recovering an unsatisfiable core, typically built by reading dispatches from a solve.
#[derive(Default)]
pub struct CoreDB {
    /// The clause used to establish the formula is unsatisfiabile.
    conflict: Option<ClauseKey>,

    /// A buffer used when reading dispatches regarding a clause.
    clause_buffer: Vec<abLiteral>,

    /// A buffer used when reading dispatches regarding an instance of resulution.
    ///
    /// [ClauseKey]s are keys to those clauses used during resolution.
    resolution_buffer: Vec<ClauseKey>,

    /// A queue of all clauses used during resolution.
    resolution_q: VecDeque<Vec<ClauseKey>>,

    /// A buffer to store a dispatched instance of BCP.
    bcp_buffer: Option<(ClauseKey, abLiteral)>,

    /// A map of clause keys to clauses of the original formula.
    original_map: HashMap<ClauseKey, vClause>,

    clause_map: HashMap<ClauseKey, Vec<ClauseKey>>,

    literal_map: HashMap<abLiteral, Vec<ClauseKey>>,
}

impl CoreDB {
    pub fn core_clauses(&self) -> Result<Vec<vClause>, err::Core> {
        let mut core_q = std::collections::VecDeque::<ClauseKey>::new();
        let mut seen_keys = std::collections::BTreeSet::new();
        let mut seen_literals = std::collections::BTreeSet::new();
        let mut core_clauses = std::collections::BTreeSet::new();

        // start with the conflict, then loop
        match self.conflict {
            Some(c) => core_q.push_back(c),
            None => return Err(err::Core::NoConflict),
        }

        /*
        key set ensures processing only happens on a fresh key

        if the key is for a formula, then clause is recorded and the literals of the clause are checked against the observed literals
        otherwise, the clauses used when resolving the learnt clause are added

         when checking literals, if the negation of the literal has been observed at level 0 then it was relevant to the conflict
         so, if the literal was obtained either by resolution or directly from some clause, then that clause or the clauses used for resolution are added to the q
         this skips assumed literals
         */
        'the_loop: while let Some(key) = core_q.pop_front() {
            if !seen_keys.insert(key) {
                continue 'the_loop;
            }
            let maybe_clause = match key {
                ClauseKey::Unit(_) => {
                    // todo: fixup
                    todo!()
                }
                ClauseKey::Original(_) => match self.original_map.get(&key) {
                    Some(the_clause) => Some(the_clause),
                    None => return Err(err::Core::MissedKey),
                },

                ClauseKey::Binary(_) => match self.clause_map.get(&key) {
                    None => match self.original_map.get(&key) {
                        Some(the_clause) => Some(the_clause),
                        None => return Err(err::Core::MissedKey),
                    },
                    Some(keys) => {
                        core_q.extend(keys);
                        None
                    }
                },

                ClauseKey::Addition(_, _) => match self.clause_map.get(&key) {
                    None => return Err(err::Core::MissedKey),
                    Some(keys) => {
                        core_q.extend(keys);
                        None
                    }
                },
            };

            if let Some(clause) = maybe_clause {
                'literal_loop: for literal in clause.literals() {
                    if !seen_literals.insert(*literal) {
                        continue 'literal_loop;
                    } else if let Some(past) = self.literal_map.get(&literal.negate()) {
                        core_q.extend(past)
                    }
                }
                core_clauses.insert(clause);
            }
        }

        Ok(core_clauses.into_iter().cloned().collect())
    }
}

impl CoreDB {
    pub fn process_resolution_delta(&mut self, δ: &Resolution) -> Result<(), err::Core> {
        use delta::Resolution::*;
        match δ {
            Begin => {
                if !self.resolution_buffer.is_empty() {
                    return Err(err::Core::CorruptClauseBuffer);
                }
            }
            End => {
                let the_clause = std::mem::take(&mut self.resolution_buffer);
                self.resolution_q.push_back(the_clause)
            }
            Used(k) => self.resolution_buffer.push(*k),
            Subsumed(_, _) => {} // TODO: maybe
        }
        Ok(())
    }

    pub fn process_clause_db_delta(&mut self, δ: &ClauseDB) -> Result<(), err::Core> {
        use delta::ClauseDB::*;
        match δ {
            ClauseStart => {
                if !self.resolution_buffer.is_empty() {
                    return Err(err::Core::CorruptClauseBuffer);
                }
            }

            ClauseLiteral(literal) => {
                self.clause_buffer.push(*literal);
            }

            Added(key) | Transfer(_, key) => {
                let Some(the_sources) = self.resolution_q.pop_front() else {
                    return Err(err::Core::QueueMiss);
                };
                self.clause_map.insert(*key, the_sources);
                self.clause_buffer.clear();
            }
            Original(key) => {
                let the_clause = std::mem::take(&mut self.clause_buffer);
                self.original_map.insert(*key, the_clause);
            }
            Deletion(_) => {
                self.clause_buffer.clear();
            }

            BCP(_) => {}
        }
        Ok(())
    }

    pub fn process_literal_db_delta(&mut self, _δ: &LiteralDB) -> Result<(), err::Core> {
        // TODO: unit resolution
        // use delta::LiteralDB::*;
        // match δ {
        //     Assumption(_) => {}
        //     ResolutionProof(literal) => {
        //         let Some(the_sources) = self.resolution_q.pop_front() else {
        //             return Err(err::Core::QueueMiss);
        //         };
        //         self.literal_map.insert(*literal, the_sources);
        //     }
        //     Proof(_) => {
        //         let Some((clause, to)) = self.bcp_buffer.take() else {
        //             return Err(err::Core::EmptyBCPBuffer);
        //         };
        //         self.literal_map.insert(to, vec![clause]);
        //     }
        //     Forced(key, literal) => {
        //         self.literal_map.insert(*literal, vec![*key]);
        //     }
        // }
        Ok(())
    }

    pub fn process_atom_db_delta(&mut self, δ: &AtomDB) -> Result<(), err::Core> {
        use delta::AtomDB::*;
        match δ {
            Unsatisfiable(key) => self.conflict = Some(*key),
            _ => {}
        };
        Ok(())
    }

    pub fn process_bcp_delta(&mut self, δ: &BCP) -> Result<(), err::Core> {
        use delta::BCP::*;
        match δ {
            Instance {
                clause: via,
                literal: to,
            } => self.bcp_buffer = Some((*via, *to)),
            Conflict { .. } => {}
        }

        Ok(())
    }
}

type CoreReceiver<'g> = Box<dyn FnMut(&Dispatch) -> Result<(), err::Core> + 'g>;

/* This is fairly gnarly.
  The goal is to acquire a single lock on a core database for the duration of the returned function.
  This, while also allowing for the core database to be optional within the context of whatever calls this.
  Simplest, then, seems to be passing in the core database as it would be found in the context…
*/
/// Builds a database for recovering an unsatisfiable core of a formula.
///
/// Locks the database.
/// Useful for cli to be optional.
/// Besides, unsatisfiable and distinct thread, so little is lost.
/// Specifically, it is almost certainly the case no further dispatches would be of interest on a thread with a core.
#[allow(clippy::single_match)]
#[allow(clippy::collapsible_match)]
pub fn core_db_builder(core_db_ptr: &Option<Arc<Mutex<CoreDB>>>) -> CoreReceiver {
    let mut core_db = core_db_ptr.as_ref().unwrap().lock().unwrap();
    let handler = move |dispatch: &Dispatch| {
        match dispatch {
            Dispatch::Delta(the_delta) => {
                use Delta::*;
                match the_delta {
                    //
                    Resolution(δ) => core_db.process_resolution_delta(δ)?,

                    ClauseDB(δ) => core_db.process_clause_db_delta(δ)?,

                    LiteralDB(δ) => core_db.process_literal_db_delta(δ)?,

                    AtomDB(δ) => core_db.process_atom_db_delta(δ)?,

                    BCP(δ) => core_db.process_bcp_delta(δ)?,
                }
            }
            _ => {}
        }
        Ok(())
    };
    Box::new(handler)
}
