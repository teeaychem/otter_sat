use crate::{
    context::store::ClauseKey,
    context::Context,
    structures::{
        clause::stored::WatchStatus,
        literal::{Literal, LiteralSource},
        variable::list::VariableList,
    },
};

pub enum PropagationInfo {
    BinaryQueue(ClauseKey),
    BinaryInspection(ClauseKey),
    LongQueue(ClauseKey),
    LongInspection(ClauseKey),
}

use crate::log::targets::PROPAGATION as LOG_PROPAGATION;

use super::WatchElement;
impl Context {
    pub fn BCP(&mut self, literal: Literal) -> Result<(), ClauseKey> {
        unsafe {
            let the_variable = self.variables.get_unsafe(literal.index());

            let binary_list = match literal.polarity() {
                true => &mut *the_variable.negative_occurrences_binary.get(),
                false => &mut *the_variable.positive_occurrences_binary.get(),
            };

            for element in binary_list {
                let WatchElement::Binary(check, clause_key) = element else {
                    log::error!(target: LOG_PROPAGATION, "Long clause found in binary watch list.");
                    panic!("Corrupt watch list")
                };

                match self.variables.value_of(check.index()) {
                    None => match self.q_literal(*check, LiteralSource::BCP(*clause_key)) {
                        Ok(()) => {}
                        Err(_key) => {
                            log::trace!(target: LOG_PROPAGATION, "Queueing consueqnece of {clause_key} {literal} failed.");
                            return Err(*clause_key);
                        }
                    },
                    Some(value) if check.polarity() != value => {
                        log::trace!(target: LOG_PROPAGATION, "Inspecting consueqnece of {clause_key} {literal} failed.");
                        return Err(*clause_key);
                    }
                    Some(_) => {
                        log::trace!(target: LOG_PROPAGATION, "Missed implication of {clause_key} {literal}.");
                        // a missed implication, as this is binary
                    }
                }
            }

            // reborrow required…
            let the_variable = self.variables.get_unsafe(literal.index());

            let list = match literal.polarity() {
                true => &mut *the_variable.negative_occurrences.get(),
                false => &mut *the_variable.positive_occurrences.get(),
            };

            let mut index = 0;
            let mut length = list.len();

            'long_loop: while index < length {
                let WatchElement::Clause(clause_key) = list.get_unchecked(index) else {
                    log::error!(target: LOG_PROPAGATION, "Binary clause found in long watch list.");
                    panic!("Corrupt watch list")
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
                        panic!("Corrupt watch list")
                    }
                    Ok(WatchStatus::Witness) | Ok(WatchStatus::None) => {
                        list.swap_remove(index);
                        length -= 1;
                        continue 'long_loop;
                    }
                    Ok(WatchStatus::Conflict) | Ok(WatchStatus::TwoConflict) => {
                        log::error!(target: LOG_PROPAGATION, "Conflict from updating watch during propagation.");
                        panic!("Corrupt watch list")
                    }
                    Err(()) => {
                        let the_watch = *clause.get_unchecked(0);
                        // assert_ne!(the_watch.index(), literal.index());
                        let watch_value = self.variables.value_of(the_watch.index());
                        match watch_value {
                            Some(value) if the_watch.polarity() != value => {
                                log::trace!(target: LOG_PROPAGATION, "Inspecting consueqnece of {clause_key} {literal} failed.");
                                return Err(*clause_key);
                            }
                            None => {
                                match self.q_literal(the_watch, LiteralSource::BCP(*clause_key)) {
                                    Ok(()) => {}
                                    Err(_key) => {
                                        log::trace!(target: LOG_PROPAGATION, "Queuing consueqnece of {clause_key} {literal} failed.");
                                        return Err(*clause_key);
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
