use std::borrow::Borrow;

use crate::{
    context::Context,
    db::{clause::ClauseKind, variable::watch_db::WatchElement},
    dispatch::{
        library::delta::{self, Delta},
        Dispatch,
    },
    misc::log::targets::{self},
    structures::literal::{Literal, LiteralT},
    types::{
        err::{self},
        gen::{self},
    },
};

impl Context {
    /// # Safety
    /// BCP extends the change of status of the given literal to the literals in the relevant watch list
    /// Mutable access to distinct literals.
    /// Work through two lists, which *from the perspective of the compiler* could contain the same literal.
    /// However, this will never be the case
    pub unsafe fn bcp(&mut self, literal: impl Borrow<Literal>) -> Result<(), err::BCP> {
        let literal = literal.borrow();
        let binary_list = &mut *self.variable_db.watch_list(
            literal.var(),
            ClauseKind::Binary,
            !literal.polarity(),
        );

        for element in binary_list {
            let WatchElement::Binary(check, clause_key) = element else {
                log::error!(target: targets::PROPAGATION, "Long clause found in binary watch list.");
                return Err(err::BCP::CorruptWatch);
            };

            match self.variable_db.value_of(check.var()) {
                None => match self.q_literal(*check) {
                    Ok(gen::Queue::Qd) => {
                        if let Some(dispatcher) = &self.dispatcher {
                            let delta = delta::BCP::Instance {
                                from: *literal,
                                via: *clause_key,
                                to: *check,
                            };
                            dispatcher(Dispatch::Delta(Delta::BCP(delta)));
                        }
                        self.literal_db
                            .record_literal(check, gen::src::Literal::BCP(*clause_key));
                    }
                    Err(_key) => {
                        return Err(err::BCP::Conflict(*clause_key));
                    }
                },
                Some(value) if check.polarity() != value => {
                    log::trace!(target: targets::PROPAGATION, "Consequence of {clause_key} and {literal} is contradiction.");
                    if let Some(dispatcher) = &self.dispatcher {
                        let delta = delta::BCP::Conflict {
                            from: *literal,
                            via: *clause_key,
                        };
                        dispatcher(Dispatch::Delta(Delta::BCP(delta)));
                    }
                    return Err(err::BCP::Conflict(*clause_key));
                }
                Some(_) => {
                    log::trace!(target: targets::PROPAGATION, "Missed implication of {clause_key} {literal}.");
                    // a missed implication, as this is binary
                }
            }
        }

        let list =
            &mut *self
                .variable_db
                .watch_list(literal.var(), ClauseKind::Long, !literal.polarity());

        let mut index = 0;
        let mut length = list.len();

        'long_loop: while index < length {
            let WatchElement::Clause(clause_key) = list.get_unchecked(index) else {
                log::error!(target: targets::PROPAGATION, "Binary clause found in long watch list.");
                return Err(err::BCP::CorruptWatch);
            };

            /*
            TODO: From the FRAT paper neither MiniSAT nor CaDiCaL store clause identifiers
            So, there may be some way to avoid this… unless there's a NULLPTR check or…
             */
            let clause = match self.clause_db.get_carefully_mut(*clause_key) {
                Some(stored_clause) => stored_clause,
                None => {
                    list.swap_remove(index);
                    length -= 1;
                    continue 'long_loop;
                }
            };

            match clause.update_watch(literal, &mut self.variable_db) {
                Ok(gen::Watch::Witness) | Ok(gen::Watch::None) => {
                    list.swap_remove(index);
                    length -= 1;
                    continue 'long_loop;
                }
                Ok(gen::Watch::Conflict) => {
                    log::error!(target: targets::PROPAGATION, "Conflict from updating watch during propagation.");
                    return Err(err::BCP::CorruptWatch);
                }
                Err(()) => {
                    let the_watch = *clause.get_unchecked(0);
                    // assert_ne!(the_watch.index(), literal.index());
                    let watch_value = self.variable_db.value_of(the_watch.var());
                    match watch_value {
                        Some(value) if the_watch.polarity() != value => {
                            self.clause_db.note_use(*clause_key);
                            if let Some(dispatcher) = &self.dispatcher {
                                let delta = delta::BCP::Conflict {
                                    from: *literal,
                                    via: *clause_key,
                                };
                                dispatcher(Dispatch::Delta(Delta::BCP(delta)));
                            }
                            return Err(err::BCP::Conflict(*clause_key));
                        }
                        None => {
                            self.clause_db.note_use(*clause_key);
                            let Ok(gen::Queue::Qd) = self.q_literal(the_watch) else {
                                return Err(err::BCP::Conflict(*clause_key));
                            };

                            if let Some(dispatcher) = &self.dispatcher {
                                let delta = delta::BCP::Instance {
                                    from: *literal,
                                    via: *clause_key,
                                    to: the_watch,
                                };
                                dispatcher(Dispatch::Delta(Delta::BCP(delta)));
                            }
                            self.literal_db
                                .record_literal(the_watch, gen::src::Literal::BCP(*clause_key));
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
