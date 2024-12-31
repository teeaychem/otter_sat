//! Boolean constraint propagation
//!
//! Take queued consequences and check clauses.
//!
//! The atom database, the clause database, and the consequence queue, and so indirectly the literal database.

use std::borrow::Borrow;

use crate::{
    context::Context,
    db::{
        atom::watch_db::{self, Watcher},
        clause::ClauseKind,
        consequence_q::{self},
    },
    dispatch::{
        library::delta::{self, Delta},
        Dispatch,
    },
    misc::log::targets::{self},
    structures::literal::{self, abLiteral, Literal},
    types::err::{self},
};

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

impl Context {
    /// # Safety
    /// The implementation of bcp requires a key invariant to be upheld:
    /// - Watch elements at index 0.
    ///
    pub unsafe fn bcp(&mut self, literal: impl Borrow<abLiteral>) -> Result<(), err::BCP> {
        let literal = literal.borrow();
        let binary_list = &mut *self.atom_db.get_watch_list_unchecked(
            literal.atom(),
            ClauseKind::Binary,
            !literal.polarity(),
        );

        for element in binary_list {
            let Watcher::Binary(check, clause_key) = element else {
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

        let long_list = &mut *self.atom_db.get_watch_list_unchecked(
            literal.atom(),
            ClauseKind::Long,
            !literal.polarity(),
        );

        let mut index = 0;
        let mut length = long_list.len();

        'long_loop: while index < length {
            let Watcher::Clause(clause_key) = long_list.get_unchecked(index) else {
                log::error!(target: targets::PROPAGATION, "Binary clause found in long watch list.");
                return Err(err::BCP::CorruptWatch);
            };

            /*
            TODO: From the FRAT paper neither MiniSAT nor CaDiCaL store clause identifiers
            So, there may be some way to avoid this… unless there's a NULLPTR check or…
             */
            let clause = match self.clause_db.get_db_clause_mut(clause_key) {
                Some(stored_clause) => stored_clause,
                None => {
                    long_list.swap_remove(index);
                    length -= 1;
                    continue 'long_loop;
                }
            };

            match clause.update_watch(literal.atom(), &mut self.atom_db) {
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
                    let the_watch = *clause.get_unchecked(0);
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
        Ok(())
    }
}
