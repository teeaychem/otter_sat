//! A context method to aid boolean constraint propagation
//!
//! See [GenericContext::bcp] for the relevant context method.
//!
//! # Overview
//! Propagates an atom being assigned some value, given as a literal.
//!
//! This is done by examining clauses watching the atom with the opposite polarity and updating the watches of the clause, if possible, queuing the consequence of the asserting clause, or identifying the clause conflicts with the current valuation.
//!
//! # Complications
//!
//! Use is made of [get_watch_list_unchecked](crate::db::atom::AtomDB::get_watch_list_unchecked) to obtain a pointer to watch lists.
//! A handful of issues are avoided by doing this:
//! 1. A mutable borrow of the database for a watch list conflicting with an immutable borrow of the database to obtain the value of an atom.
//! 2. A mutable borrow of the context conflicting with a mutable borrow to add a literal to the consequence queue.
//! 3. A mutable borrow of the database in a call to update the watched literals in some clause.
//!
//! (1) and (2) could be avoided by a more nuanced borrow checker, as these are separate structures, combined to ease reasoning about the library.
//! This is not the case for (3), as a watch list has been borrowed, and a call to [dbClause::update_watch](crate::db::clause::db_clause::dbClause::update_watch) may mutate watch lists.
//! Still, the *borrowed* watch list will not be mutated.
//! For, the literal bcp is being called on has been given some value, and the inspected list being for the atom with the opposite value.
//! And, the atom with the opposite value is not a [candidate](crate::db::clause::db_clause::dbClause) for updating a watch to as it:
//! - Has some value.
//! - Has a value which conflicts with the current valuation.
//!
//! # Heuristics
//!
//! Propagation happens in two steps, distinguished by clauses length:
//! - First, with respect to binary clauses.
//! - Second, with respect to long clauses.
//!
//! This sequence is motivated by various considerations.
//! For example, binary clauses always have an lbd of at most 2, binary clauses do not require accessing the clause database and updating watches, etc.
//!
//! # Example
//!
//! bcp is a mutating method, and a typical application will match against the result of the mutation.
//! For example, a conflict may lead to conflict analysis and no conflict may lead to a decision being made.
//!
//! ```rust,ignore
//! match self.bcp(literal) {
//!     Err(err::BCP::Conflict(key)) => {
//!         if self.literal_db.decision_made() {
//!             let analysis_result = self.conflict_analysis(&clause_key)?;
//!             ...
//!         }
//!     }
//!     ...
//!     Ok => {
//!         match self.make_decision()? {
//!             ...
//!         }
//!     }
//! }
//! ```
use std::borrow::Borrow;

use crate::{
    context::GenericContext,
    db::{
        atom::watch_db::{self, WatchTag},
        consequence_q::{self},
    },
    dispatch::{
        library::delta::{self, Delta},
        macros::{self},
        Dispatch,
    },
    misc::log::targets::{self},
    structures::{
        clause::ClauseKind,
        literal::{self, abLiteral, Literal},
    },
    types::err::{self},
};

impl<R: rand::Rng + std::default::Default> GenericContext<R> {
    /// For documentation see [procedures::bcp](crate::procedures::bcp).
    /// # Safety
    /// The implementation of bcp requires a key invariant to be upheld:
    /// <div class="warning">
    /// The literal at index 0 is a watched literal.
    /// </div>
    pub unsafe fn bcp(&mut self, literal: impl Borrow<abLiteral>) -> Result<(), err::BCP> {
        let literal = literal.borrow();
        let decision_level = self.literal_db.decision_count();

        // Binary clauses block.
        {
            // Note, this does not require updating watches.
            let binary_list = &mut *self.atom_db.get_watch_list_unchecked(
                literal.atom(),
                ClauseKind::Binary,
                !literal.polarity(),
            );

            for element in binary_list {
                let WatchTag::Binary(check, clause_key) = element else {
                    log::error!(target: targets::PROPAGATION, "Long clause found in binary watch list.");
                    return Err(err::BCP::CorruptWatch);
                };

                match self.atom_db.value_of(check.atom()) {
                    None => {
                        match self.value_and_queue(
                            *check,
                            consequence_q::QPosition::Back,
                            decision_level,
                        ) {
                            Ok(consequence_q::Ok::Qd) => {
                                macros::dispatch_bcp_delta!(self, Instance, *check, *clause_key);
                                self.record_literal(check, literal::Source::BCP(*clause_key));
                            }

                            Ok(consequence_q::Ok::Skip) => {}

                            Err(_key) => {
                                return Err(err::BCP::Conflict(*clause_key));
                            }
                        }
                    }

                    Some(value) if check.polarity() != value => {
                        // Note the conflict
                        log::trace!(target: targets::PROPAGATION, "Consequence of {clause_key} and {literal} is contradiction.");
                        macros::dispatch_bcp_delta!(self, Conflict, *literal, *clause_key);

                        return Err(err::BCP::Conflict(*clause_key));
                    }

                    Some(_) => {
                        log::trace!(target: targets::PROPAGATION, "Repeat implication of {clause_key} {literal}.");
                        // a repeat implication, as this is binary
                    }
                }
            }
        }

        // Long clause block.
        {
            let long_list = &mut *self.atom_db.get_watch_list_unchecked(
                literal.atom(),
                ClauseKind::Long,
                !literal.polarity(),
            );

            let mut index = 0;
            let mut length = long_list.len();

            'long_loop: while index < length {
                let WatchTag::Clause(clause_key) = long_list.get_unchecked(index) else {
                    log::error!(target: targets::PROPAGATION, "Binary clause found in long watch list.");
                    return Err(err::BCP::CorruptWatch);
                };

                // TODO: From the FRAT paper neither MiniSAT nor CaDiCaL store clause identifiers.
                // So, there may be some way to avoid this… unless there's a NULLPTR check or…
                let db_clause = match self.clause_db.get_mut(clause_key) {
                    Ok(stored_clause) => stored_clause,
                    Err(_) => {
                        long_list.swap_remove(index);
                        length -= 1;
                        continue 'long_loop;
                    }
                };

                match db_clause.update_watch(literal.atom(), &mut self.atom_db) {
                    Ok(watch_db::WatchStatus::Witness) | Ok(watch_db::WatchStatus::None) => {
                        long_list.swap_remove(index);
                        length -= 1;
                        continue 'long_loop;
                    }

                    Ok(watch_db::WatchStatus::Conflict) => {
                        log::error!(target: targets::PROPAGATION, "Conflict from updating watch during propagation.");
                        return Err(err::BCP::CorruptWatch);
                    }

                    Err(()) => {
                        // After the call to update_watch, any atom without a value will be in position 0.
                        let the_watch = *db_clause.get_unchecked(0);
                        let watch_value = self.atom_db.value_of(the_watch.atom());

                        match watch_value {
                            Some(value) if the_watch.polarity() != value => {
                                self.clause_db.note_use(*clause_key);
                                macros::dispatch_bcp_delta!(self, Conflict, *literal, *clause_key);

                                return Err(err::BCP::Conflict(*clause_key));
                            }

                            None => {
                                self.clause_db.note_use(*clause_key);

                                match self.value_and_queue(
                                    the_watch,
                                    consequence_q::QPosition::Back,
                                    decision_level,
                                ) {
                                    Ok(consequence_q::Ok::Qd) | Ok(consequence_q::Ok::Skip) => {}

                                    Err(_) => return Err(err::BCP::Conflict(*clause_key)),
                                };

                                macros::dispatch_bcp_delta!(self, Instance, the_watch, *clause_key);
                                self.record_literal(the_watch, literal::Source::BCP(*clause_key));
                            }

                            Some(_) => {}
                        }
                    }
                }

                index += 1;
                continue 'long_loop;
            }
        }
        Ok(())
    }
}
