use crate::context::{
    level::{Level, LevelIndex},
    store::{ClauseKey, ClauseStore},
};
use crate::structures::variable::list::VariableList;
use crate::structures::{
    clause::stored::WatchStatus,
    literal::{Literal, LiteralSource},
    variable::{
        delegate::{push_back_consequence, VariableStore},
        Variable, VariableId,
    },
};

use std::cell::UnsafeCell;

use super::WatchElement;

impl Variable {
    pub fn new(id: VariableId) -> Self {
        Self {
            decision_level: UnsafeCell::new(None),
            id,
            value: UnsafeCell::new(None),
            previous_value: UnsafeCell::new(None),
            positive_occurrences: UnsafeCell::new(Vec::with_capacity(512)),
            positive_occurrences_binary: UnsafeCell::new(Vec::with_capacity(512)),
            negative_occurrences: UnsafeCell::new(Vec::with_capacity(512)),
            negative_occurrences_binary: UnsafeCell::new(Vec::with_capacity(512)),
        }
    }

    pub fn index(&self) -> usize {
        self.id as usize
    }

    pub fn decision_level(&self) -> Option<LevelIndex> {
        unsafe { *self.decision_level.get() }
    }

    pub const fn id(&self) -> VariableId {
        self.id
    }

    pub fn watch_added(&self, element: WatchElement, polarity: bool) {
        unsafe {
            match element {
                WatchElement::Binary(_, _) => match polarity {
                    true => (*self.positive_occurrences_binary.get()).push(element),
                    false => {
                        (*self.negative_occurrences_binary.get()).push(element);
                    }
                },
                WatchElement::Clause(_) => match polarity {
                    true => (*self.positive_occurrences.get()).push(element),
                    false => {
                        (*self.negative_occurrences.get()).push(element);
                    }
                },
            }
        }
    }

    pub fn watch_removed(&self, clause_key: ClauseKey, polarity: bool) {
        match polarity {
            true => unsafe {
                let list = &mut *self.positive_occurrences.get();
                list.retain(|element| match element {
                    WatchElement::Binary(_, _) => true,
                    WatchElement::Clause(key) if *key != clause_key => true,
                    WatchElement::Clause(_) => false,
                });
            },
            false => unsafe {
                let list = &mut *self.negative_occurrences.get();
                list.retain(|element| match element {
                    WatchElement::Binary(_, _) => true,
                    WatchElement::Clause(key) if *key != clause_key => true,
                    WatchElement::Clause(_) => false,
                });
            },
        };
    }

    pub fn value(&self) -> Option<bool> {
        unsafe { *self.value.get() }
    }

    pub fn previous_value(&self) -> Option<bool> {
        unsafe { *self.previous_value.get() }
    }

    pub fn set_value(&self, polarity: Option<bool>, level: Option<LevelIndex>) {
        unsafe {
            *self.previous_value.get() = *self.value.get();
            *self.value.get() = polarity;
            *self.decision_level.get() = level
        }
    }
}

/*
Placed here for access to the occurrence lists of a variable
*/
pub fn propagate_literal(
    literal: Literal,
    variables: &mut VariableStore,
    clause_store: &mut ClauseStore,
    level: &mut Level,
) -> Result<(), ClauseKey> {
    unsafe {
        let the_variable = variables.get_unsafe(literal.index());

        let binary_list = match literal.polarity() {
            true => &mut *the_variable.negative_occurrences_binary.get(),
            false => &mut *the_variable.positive_occurrences_binary.get(),
        };

        for element in binary_list {
            match element {
                WatchElement::Clause(_) => {
                    panic!("binary clause occurence list contains non-binary clause")
                }
                WatchElement::Binary(check, clause_key) => {
                    match variables.value_of(check.index()) {
                        None => match push_back_consequence(
                            variables,
                            *check,
                            LiteralSource::Propagation(*clause_key),
                            level,
                        ) {
                            Ok(()) => {}
                            Err(key) => return Err(key),
                        },
                        Some(polarity) if polarity == check.polarity() => {}
                        Some(_) => return Err(*clause_key),
                    }
                }
            }
        }

        // reborrow required…
        let the_variable = variables.get_unsafe(literal.index());

        let list = match literal.polarity() {
            true => &mut *the_variable.negative_occurrences.get(),
            false => &mut *the_variable.positive_occurrences.get(),
        };

        let mut index = 0;
        let mut length = list.len();

        'propagation_loop: while index < length {
            match list.get_unchecked(index) {
                WatchElement::Clause(clause_key) => {
                    let clause = match clause_store.get_carefully_mut(*clause_key) {
                        Some(stored_clause) => stored_clause,
                        None => {
                            list.swap_remove(index);
                            length -= 1;
                            continue 'propagation_loop;
                        }
                    };

                    match clause.update_watch(literal, variables) {
                        Ok(WatchStatus::TwoWitness) | Ok(WatchStatus::TwoNone) => {
                            index += 1;
                            continue 'propagation_loop;
                        }
                        Ok(WatchStatus::Witness) | Ok(WatchStatus::None) => {
                            list.swap_remove(index);
                            length -= 1;
                            continue 'propagation_loop;
                        }
                        Ok(_) => panic!("can't get conflict from update"),
                        Err(()) => {
                            let the_watch = clause.get_unchecked(0);
                            match variables.value_of(the_watch.index()) {
                                Some(value) if the_watch.polarity() != value => {
                                    return Err(*clause_key);
                                }
                                None => {
                                    match push_back_consequence(
                                        variables,
                                        *the_watch,
                                        LiteralSource::Propagation(*clause_key),
                                        level,
                                    ) {
                                        Ok(()) => {}
                                        Err(key) => return Err(key),
                                    };
                                }
                                Some(_) => {}
                            }
                        }
                    }
                }
                WatchElement::Binary(_, _) => {
                    panic!("clause occurence list contains binary clause")
                }
            }

            index += 1;
            continue 'propagation_loop;
        }
    }
    Ok(())
}
