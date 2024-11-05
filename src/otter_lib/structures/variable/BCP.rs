use crate::{
    context::stores::ClauseKey,
    context::Context,
    structures::{
        clause::stored::WatchStatus,
        literal::{Literal, LiteralSource},
        variable::list::VariableList,
    },
};

use crate::log::targets::PROPAGATION as LOG_PROPAGATION;

pub enum BCPIssue {
    Conflict(ClauseKey),
    CorruptWatch,
}

use super::WatchElement;
impl Context {
    pub fn BCP(&mut self, literal: Literal) -> Result<(), BCPIssue> {
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
                    return Err(BCPIssue::CorruptWatch);
                };

                match self.variables.value_of(check.index()) {
                    None => match self.q_literal(*check, LiteralSource::BCP(*clause_key)) {
                        Ok(()) => {}
                        Err(_key) => {
                            log::trace!(target: LOG_PROPAGATION, "Queueing consequence of {clause_key} {literal} failed.");
                            return Err(BCPIssue::Conflict(*clause_key));
                        }
                    },
                    Some(value) if check.polarity() != value => {
                        log::trace!(target: LOG_PROPAGATION, "Inspecting consequence of {clause_key} {literal} failed.");
                        return Err(BCPIssue::Conflict(*clause_key));
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
                    return Err(BCPIssue::CorruptWatch);
                };

                let clause = match self.clause_store.get_carefully_mut(*clause_key) {
                    Some(stored_clause) => stored_clause,
                    None => {
                        list.swap_remove(index);
                        length -= 1;
                        continue 'long_loop;
                    }
                };

                match clause.update_watch(literal, &mut self.variables) {
                    Ok(WatchStatus::TwoWitness) | Ok(WatchStatus::TwoNone) => {
                        log::error!(target: LOG_PROPAGATION, "Length two clause found in long list.");
                        println!("here");
                        return Err(BCPIssue::CorruptWatch);
                    }
                    Ok(WatchStatus::Witness) | Ok(WatchStatus::None) => {
                        list.swap_remove(index);
                        length -= 1;
                        continue 'long_loop;
                    }
                    Ok(WatchStatus::Conflict) | Ok(WatchStatus::TwoConflict) => {
                        log::error!(target: LOG_PROPAGATION, "Conflict from updating watch during propagation.");
                        return Err(BCPIssue::CorruptWatch);
                    }
                    Err(()) => {
                        let the_watch = *clause.get_unchecked(0);
                        // assert_ne!(the_watch.index(), literal.index());
                        let watch_value = self.variables.value_of(the_watch.index());
                        match watch_value {
                            Some(value) if the_watch.polarity() != value => {
                                log::trace!(target: LOG_PROPAGATION, "Inspecting consequence of {clause_key} {literal} failed.");
                                return Err(BCPIssue::Conflict(*clause_key));
                            }
                            None => {
                                match self.q_literal(the_watch, LiteralSource::BCP(*clause_key)) {
                                    Ok(()) => {}
                                    Err(_) => {
                                        log::trace!(target: LOG_PROPAGATION, "Queuing consequence of {clause_key} {literal} failed.");
                                        return Err(BCPIssue::Conflict(*clause_key));
                                    }
                                };
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
