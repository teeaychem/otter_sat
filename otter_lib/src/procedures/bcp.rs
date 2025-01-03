//! A context method to aid boolean constraint propagation
//!
//! See [GenericContext::bcp] for details.

use std::borrow::Borrow;

use crate::{
    context::GenericContext,
    db::{
        atom::watch_db::{self, WatchTag},
        consequence_q::{self},
    },
    dispatch::{
        library::delta::{self, Delta},
        Dispatch,
    },
    misc::log::targets::{self},
    structures::{
        clause::ClauseKind,
        literal::{self, abLiteral, Literal},
    },
    types::err::{self},
};

/// A macro to simplify dispatches.
macro_rules! send {
    ($self:ident, $dispatcher:ident, $variant:ident, $from:expr, $via:expr ) => {{
        if let Some(dispatcher) = &$self.$dispatcher {
            let delta = delta::BCP::$variant {
                literal: $from,
                clause: $via,
            };
            dispatcher(Dispatch::Delta(Delta::BCP(delta)));
        }
    }};
}

impl<R: rand::Rng + std::default::Default> GenericContext<R> {
    /// Propagates an atom being assigned some value, given as a literal.
    ///
    /// This is done by examining clauses watching the atom with the opposite polarity and updating the watches of the clause, if possible, queuing the consequence of the asserting clause, or identifying the clause conflicts with the current valuation.
    ///
    /// ```rust,ignore
    /// match unsafe { self.bcp(literal) } {
    ///   Err(err::BCP::Conflict(key)) => {
    ///     if self.literal_db.choice_made() {
    ///       let analysis_result = self.conflict_analysis(&clause_key)?;
    ///       ...
    ///     }
    ///   ...
    ///   }
    /// }
    /// ```
    /// # Safety
    /// The implementation of bcp requires a key invariant to be upheld:
    /// - Watch elements at index 0.
    ///
    /// Further, use is made of [get_watch_list_unchecked](crate::db::atom::AtomDB::get_watch_list_unchecked) to obtain a pointer to watch lists.
    /// A handful of issues are avoided by doing this:
    /// 1. A mutable borrow of the database for a watch list conflicting with an immutable borrow of the database to obtain the value of an atom.
    /// 2. A mutable borrow of the context conflicting with a mutable borrow to add a literal to the consequence queue.
    /// 3. A mutable borrow of the database in a call to update the watched literals in some clause.
    ///
    /// (1) and (2) could be avoided by a more nuanced borrow checker, as these are separate structures, combined to ease reasoning about the library.
    /// This is not the case for (3), as a watch list has been borrowed, and a call to [dbClause::update_watch](crate::db::clause::db_clause::dbClause::update_watch) may mutate watch lists.
    /// Still, the *borrowed* watch list will not be mutated.
    /// For, the literal bcp is being called on has been given some value, and the inspected list being for the atom with the opposite value.
    /// And, the atom with the opposite value is not a [candidate](crate::db::clause::db_clause::dbClause) for updating a watch to as it:
    /// - Has some value.
    /// - Has a value which conflicts with the current valuation.
    pub unsafe fn bcp(&mut self, literal: impl Borrow<abLiteral>) -> Result<(), err::BCP> {
        let literal = literal.borrow();

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
                    None => match self.q_literal(*check) {
                        Ok(consequence_q::Ok::Qd) => {
                            send!(self, dispatcher, Instance, *check, *clause_key);
                            self.record_literal(check, literal::Source::BCP(*clause_key));
                        }

                        Err(_key) => {
                            return Err(err::BCP::Conflict(*clause_key));
                        }
                    },

                    Some(value) if check.polarity() != value => {
                        // Note the conflict
                        log::trace!(target: targets::PROPAGATION, "Consequence of {clause_key} and {literal} is contradiction.");
                        send!(self, dispatcher, Conflict, *literal, *clause_key);

                        return Err(err::BCP::Conflict(*clause_key));
                    }

                    Some(_) => {
                        log::trace!(target: targets::PROPAGATION, "Missed implication of {clause_key} {literal}.");
                        // a missed implication, as this is binary
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
                let db_clause = match self.clause_db.get_db_clause_mut(clause_key) {
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
                        let the_watch = *db_clause.get_unchecked(0);
                        let watch_value = self.atom_db.value_of(the_watch.atom());

                        match watch_value {
                            Some(value) if the_watch.polarity() != value => {
                                self.clause_db.note_use(*clause_key);
                                send!(self, dispatcher, Conflict, *literal, *clause_key);

                                return Err(err::BCP::Conflict(*clause_key));
                            }

                            None => {
                                self.clause_db.note_use(*clause_key);

                                match self.q_literal(the_watch) {
                                    Ok(consequence_q::Ok::Qd) => {}
                                    Err(_) => return Err(err::BCP::Conflict(*clause_key)),
                                };

                                send!(self, dispatcher, Instance, the_watch, *clause_key);
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
