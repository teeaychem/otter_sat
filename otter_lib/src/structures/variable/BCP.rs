use std::borrow::Borrow;

use crate::{
    context::Context,
    db::{keys::ClauseKey, variable::QStatus},
    structures::{
        literal::{Literal, LiteralSource, LiteralTrait},
        variable::list::VariableList,
    },
    types::clause::WatchStatus,
};

use crate::log::targets::PROPAGATION as LOG_PROPAGATION;

pub enum BCPErr {
    Conflict(ClauseKey),
    CorruptWatch,
}

use super::WatchElement;
impl Context {
    pub fn BCP<L: Borrow<Literal>>(&mut self, literal: L) -> Result<(), BCPErr> {
        let literal = literal.borrow();
        unsafe {
            let binary_list = match literal.polarity() {
                true => &mut *self
                    .variables
                    .get_unsafe(literal.index())
                    .negative_occurrences_binary
                    .get(),
                false => &mut *self
                    .variables
                    .get_unsafe(literal.index())
                    .positive_occurrences_binary
                    .get(),
            };

            for element in binary_list {
                let WatchElement::Binary(check, clause_key) = element else {
                    log::error!(target: LOG_PROPAGATION, "Long clause found in binary watch list.");
                    return Err(BCPErr::CorruptWatch);
                };

                match self.variables.value_of(*check) {
                    None => match self.q_literal(*check) {
                        Ok(QStatus::Qd) => {
                            self.note_literal(check.canonical(), LiteralSource::BCP(*clause_key));
                        }
                        Err(_key) => {
                            log::trace!(target: LOG_PROPAGATION, "Queueing consequence of {clause_key} {literal} failed.");
                            return Err(BCPErr::Conflict(*clause_key));
                        }
                    },
                    Some(value) if check.polarity() != value => {
                        log::trace!(target: LOG_PROPAGATION, "Consequence of {clause_key} and {literal} is contradiction.");
                        return Err(BCPErr::Conflict(*clause_key));
                    }
                    Some(_) => {
                        log::trace!(target: LOG_PROPAGATION, "Missed implication of {clause_key} {literal}.");
                        // a missed implication, as this is binary
                    }
                }
            }

            let list = match literal.polarity() {
                true => &mut *self
                    .variables
                    .get_unsafe(literal.index())
                    .negative_occurrences
                    .get(),
                false => &mut *self
                    .variables
                    .get_unsafe(literal.index())
                    .positive_occurrences
                    .get(),
            };

            let mut index = 0;
            let mut length = list.len();

            'long_loop: while index < length {
                let WatchElement::Clause(clause_key) = list.get_unchecked(index) else {
                    log::error!(target: LOG_PROPAGATION, "Binary clause found in long watch list.");
                    return Err(BCPErr::CorruptWatch);
                };

                /*
                TODO: From the FRAT paper neither MiniSAT nor CaDiCaL store clause identifiers
                So, there may be some way to avoid this… unless there's a NULLPTR check or something…
                 */
                let clause = match self.clause_db.get_carefully_mut(*clause_key) {
                    Some(stored_clause) => stored_clause,
                    None => {
                        list.swap_remove(index);
                        length -= 1;
                        continue 'long_loop;
                    }
                };

                match clause.update_watch(literal, &mut self.variables) {
                    Ok(WatchStatus::Witness) | Ok(WatchStatus::None) => {
                        list.swap_remove(index);
                        length -= 1;
                        continue 'long_loop;
                    }
                    Ok(WatchStatus::Conflict) => {
                        log::error!(target: LOG_PROPAGATION, "Conflict from updating watch during propagation.");
                        return Err(BCPErr::CorruptWatch);
                    }
                    Err(()) => {
                        let the_watch = *clause.get_unchecked(0);
                        // assert_ne!(the_watch.index(), literal.index());
                        let watch_value = self.variables.value_of(the_watch);
                        match watch_value {
                            Some(value) if the_watch.polarity() != value => {
                                log::trace!(target: LOG_PROPAGATION, "Inspecting consequence of {clause_key} {literal} failed.");
                                return Err(BCPErr::Conflict(*clause_key));
                            }
                            None => {
                                let Ok(QStatus::Qd) = self.q_literal(the_watch) else {
                                    log::trace!(target: LOG_PROPAGATION, "Queuing consequence of {clause_key} {literal} failed.");
                                    return Err(BCPErr::Conflict(*clause_key));
                                };
                                self.note_literal(
                                    the_watch.canonical(),
                                    LiteralSource::BCP(*clause_key),
                                );
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
