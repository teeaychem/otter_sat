pub mod watch_db;
use watch_db::{BinaryWatch, LongWatch, WatchDB};

use crate::{
    db::ClauseKey,
    structures::literal::{CLiteral, Literal},
    types::err,
};

#[derive(Default)]
pub struct Watches {
    pub dbs: Vec<WatchDB>,
}

impl Watches {
    /// Adds `atom` being valued `value` to the binary clause wrapped in `watch_tag`.
    ///
    /// # Safety
    /// No check is made on whether a [WatchDB] exists for the atom.
    pub unsafe fn watch_binary_unchecked(&mut self, literal: &CLiteral, watch: BinaryWatch) {
        let atom = unsafe { self.dbs.get_unchecked_mut(literal.atom() as usize) };
        match literal.polarity() {
            true => atom.positive_binary.push(watch),
            false => atom.negative_binary.push(watch),
        }
    }

    /// Adds `atom` being valued `value` to the clause wrapped in `watch_tag`.
    ///
    /// The counterpart of [unwatch_long_unchecked](AtomDB::unwatch_long_unchecked).
    ///
    /// # Safety
    /// No check is made on whether a [WatchDB] exists for the atom.
    pub unsafe fn watch_long_unchecked(&mut self, literal: &CLiteral, watch: LongWatch) {
        let atom = unsafe { self.dbs.get_unchecked_mut(literal.atom() as usize) };
        let list = match literal.polarity() {
            true => &mut atom.positive_long,
            false => &mut atom.negative_long,
        };

        list.push(watch);
    }

    /// Removes `atom` being valued `value` to the clause wrapped in `watch_tag`.
    ///
    /// The counterpart of [watch_long_unchecked](AtomDB::watch_long_unchecked).
    ///
    /// # Safety
    /// No check is made on whether a [WatchDB] exists for the atom.
    /*
    If there's a guarantee keys appear at most once, the swap remove on keys could break early.
    Note also, as this shuffles the list any heuristics on traversal order of watches is void.
     */
    pub unsafe fn unwatch_long_unchecked(
        &mut self,
        literal: CLiteral,
        key: &ClauseKey,
    ) -> Result<(), err::ClauseDBError> {
        let atom = unsafe { self.dbs.get_unchecked_mut(literal.atom() as usize) };
        match key {
            ClauseKey::Original(_) | ClauseKey::Addition(_, _) => {
                let list = match literal.polarity() {
                    true => &mut atom.positive_long,
                    false => &mut atom.negative_long,
                };

                let mut index = 0;
                let mut limit = list.len();

                while index < limit {
                    let list_key = unsafe { list.get_unchecked(index).key };

                    if &list_key == key {
                        list.swap_remove(index);
                        limit -= 1;
                    } else {
                        index += 1;
                    }
                }
                Ok(())
            }
            ClauseKey::OriginalUnit(_)
            | ClauseKey::AdditionUnit(_)
            | ClauseKey::OriginalBinary(_)
            | ClauseKey::AdditionBinary(_) => Err(err::ClauseDBError::CorruptList),
        }
    }

    /// Returns the collection of binary watched clauses for `atom` to be valued with `value`.
    ///
    /// A pointer returned to help simplify [BCP](crate::procedures::bcp), though as BCP does not mutate the list of binary clauses, the pointer is marked const.
    ///
    /// # Safety
    /// No check is made on whether a [WatchDB] exists for the atom.
    pub unsafe fn watchers_binary_unchecked(&self, literal: &CLiteral) -> *const Vec<BinaryWatch> {
        let atom = unsafe { self.dbs.get_unchecked(literal.atom() as usize) };

        match !literal.polarity() {
            true => &atom.positive_binary,
            false => &atom.negative_binary,
        }
    }

    /// Returns the collection of long watched clauses for `atom` to be valued with `value`.
    ///
    /// A mutable pointer returned to help simplify [BCP](crate::procedures::bcp).
    /// Specifically, to allow for multiple mutable borrows.
    /// As, both the watch list and valuation may be mutated during BCP.
    ///
    /// # Safety
    /// No check is made on whether a [WatchDB] exists for the atom.
    pub unsafe fn watchers_long_unchecked(&mut self, literal: &CLiteral) -> *mut Vec<LongWatch> {
        let atom = unsafe { self.dbs.get_unchecked_mut(literal.atom() as usize) };

        match !literal.polarity() {
            true => &mut atom.positive_long,
            false => &mut atom.negative_long,
        }
    }
}
