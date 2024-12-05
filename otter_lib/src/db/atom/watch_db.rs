use crate::{
    db::{atom::AtomDB, clause::ClauseKind, keys::ClauseKey},
    structures::{
        atom::Atom,
        literal::{vbLiteral, Literal},
    },
    types::err::{self},
};
use std::{borrow::Borrow, cell::UnsafeCell};

#[derive(Debug)]
pub enum WatchElement {
    Binary(vbLiteral, ClauseKey),
    Clause(ClauseKey),
}

pub(super) struct WatchDB {
    positive_binary: UnsafeCell<Vec<WatchElement>>,
    positive_long: UnsafeCell<Vec<WatchElement>>,
    negative_binary: UnsafeCell<Vec<WatchElement>>,
    negative_long: UnsafeCell<Vec<WatchElement>>,
}

impl WatchDB {
    pub(super) fn new() -> Self {
        Self {
            positive_binary: UnsafeCell::new(Vec::with_capacity(512)),
            positive_long: UnsafeCell::new(Vec::with_capacity(512)),
            negative_binary: UnsafeCell::new(Vec::with_capacity(512)),
            negative_long: UnsafeCell::new(Vec::with_capacity(512)),
        }
    }

    unsafe fn watch_added(&self, element: WatchElement, polarity: bool) {
        match element {
            WatchElement::Binary(_, _) => match polarity {
                true => (*self.positive_binary.get()).push(element),
                false => (*self.negative_binary.get()).push(element),
            },
            WatchElement::Clause(_) => match polarity {
                true => (*self.positive_long.get()).push(element),
                false => (*self.negative_long.get()).push(element),
            },
        }
    }

    /*
    Swap remove on keys
    If there's a guarantee keys appear at most once, then this could break early
    As this shuffles the list any heuristics on traversal order are affected
     */
    unsafe fn watch_removed(&self, key: ClauseKey, polarity: bool) -> Result<(), err::Watch> {
        match key {
            ClauseKey::Original(_) | ClauseKey::Addition(_, _) => {
                let list = match polarity {
                    true => &mut *self.positive_long.get(),
                    false => &mut *self.negative_long.get(),
                };
                let mut index = 0;
                let mut limit = list.len();
                while index < limit {
                    let WatchElement::Clause(list_key) = list.get_unchecked(index) else {
                        return Err(err::Watch::NotLongInLong);
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
            ClauseKey::Unit(_) | ClauseKey::Binary(_) => Err(err::Watch::NotLongInLong),
        }
    }

    fn occurrences_binary(&self, polarity: bool) -> *mut Vec<WatchElement> {
        match polarity {
            true => self.positive_binary.get(),
            false => self.negative_binary.get(),
        }
    }

    fn occurrences_long(&self, polarity: bool) -> *mut Vec<WatchElement> {
        match polarity {
            true => self.positive_long.get(),
            false => self.negative_long.get(),
        }
    }
}

impl AtomDB {
    pub unsafe fn add_watch(&mut self, literal: impl Borrow<vbLiteral>, element: WatchElement) {
        self.watch_dbs
            .get_unchecked(literal.borrow().var() as usize)
            .watch_added(element, literal.borrow().polarity());
    }

    pub unsafe fn remove_watch(
        &mut self,
        literal: impl Borrow<vbLiteral>,
        key: ClauseKey,
    ) -> Result<(), err::Watch> {
        unsafe {
            self.watch_dbs
                .get_unchecked(literal.borrow().var() as usize)
                .watch_removed(key, literal.borrow().polarity())
        }
    }

    pub unsafe fn watch_list(
        &self,
        v_idx: Atom,
        kind: ClauseKind,
        polarity: bool,
    ) -> *mut Vec<WatchElement> {
        match kind {
            ClauseKind::Unit => todo!(),
            ClauseKind::Binary => &mut *self
                .watch_dbs
                .get_unchecked(v_idx as usize)
                .occurrences_binary(polarity),
            ClauseKind::Long => self
                .watch_dbs
                .get_unchecked(v_idx as usize)
                .occurrences_long(polarity),
        }
    }
}
