use crate::{
    db::keys::{ClauseKey, DecisionIndex},
    structures::variable::{Variable, VariableId},
    types::errs::{self},
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

    pub fn decision_level(&self) -> Option<DecisionIndex> {
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
                    false => (*self.negative_occurrences_binary.get()).push(element),
                },
                WatchElement::Clause(_) => match polarity {
                    true => (*self.positive_occurrences.get()).push(element),
                    false => (*self.negative_occurrences.get()).push(element),
                },
            }
        }
    }

    /*
    Swap remove on keys
    If guarantee that key appears once then this could break early
    As this shuffles the list any heuristics on traversal order are affected
     */
    pub fn watch_removed(&self, key: ClauseKey, polarity: bool) -> Result<(), errs::Watch> {
        unsafe {
            match key {
                ClauseKey::Formula(_) | ClauseKey::Learned(_, _) => {
                    let list = match polarity {
                        true => &mut *self.positive_occurrences.get(),
                        false => &mut *self.negative_occurrences.get(),
                    };
                    let mut index = 0;
                    let mut limit = list.len();
                    while index < limit {
                        let WatchElement::Clause(list_key) = list.get_unchecked(index) else {
                            return Err(errs::Watch::BinaryInLong);
                        };

                        if *list_key == key {
                            list.swap_remove(index);
                            limit -= 1;
                        } else {
                            index += 1;
                        }
                    }
                    Ok(())
                }
                ClauseKey::Binary(_) => Err(errs::Watch::BinaryInLong),
            }
        }
    }

    pub fn value(&self) -> Option<bool> {
        unsafe { *self.value.get() }
    }

    pub fn previous_value(&self) -> Option<bool> {
        unsafe { *self.previous_value.get() }
    }

    pub fn set_value(&self, polarity: Option<bool>, level: Option<DecisionIndex>) {
        unsafe {
            *self.previous_value.get() = *self.value.get();
            *self.value.get() = polarity;
            *self.decision_level.get() = level
        }
    }
}
