use crate::{
    db::{
        atom::{
            watch_db::{WatchStatus, WatchTag},
            AtomDB,
        },
        keys::ClauseKey,
    },
    structures::{
        atom::Atom,
        literal::{abLiteral, Literal},
    },
};

use super::dbClause;

/// Methods for watched literals of a clause.
///
/// # Implementation notes
///
/// The approach to watch literals follows [Optimal implementation of watched literals and more general techniques](https://www.jair.org/index.php/jair/article/view/10839).
///
/// In short:
/// - The clause is stored using an mutable, indexable data strcutre, here a vector.
/// - A watched literal 'watch A' is kept at the first index.
/// - A watched literal 'watch B' is indentified by a mutable index.
/// - An update to the watched literals is called when the atom of one of the literals is assigned some value.
/// - When doing so, watch A is updated to be the *other* watched literal, if possible.
///   - With this, watch A will always be a literal whose atom has no value, if possible.
/// - And, after doing so the the index to watch B makes a circular sweep forward over the clause (skipping watch A) in search of a new watch candidate. The index is updated if some candidate is found, and remains unchanged otherwise.
///
/// So, there are two key invariant to keep maintained:
/// 1. The first literal and the literal at the index of watch_ptr are watch candidates, if any exist.
/// 2. After any update to the watched literals, the first literal has no value on the current valuation, if possible.
impl dbClause {
    pub unsafe fn get_watch_a(&self) -> &abLiteral {
        self.get_unchecked(0)
    }

    /// Initialises watches.
    ///
    /// # Safety
    /// As watches require two or more literals, and watch_ptr must be within the bounds of the vector, use of get_unchecked on index zero and watch_ptr is safe.
    /*
    # Note
    In order to avoid redundant literal lookup, watch candidates are noted when found.
    Still, as a watch candidate may *fail* to be found it is important to check and note the watch regardless.

     Failure for a candidate for watch A to be found implies a candidate for watch B.
     Still, this is not encoded, as failure for watch A is very unlikely.
     */
    pub fn initialise_watches(&mut self, atoms: &mut AtomDB) {
        let mut watch_a_set = false;
        for (index, literal) in self.clause.iter().enumerate() {
            let index_value = atoms.value_of(literal.atom());
            match index_value {
                None => {
                    self.note_watch(literal.atom(), literal.polarity(), atoms);
                    self.clause.swap(0, index);
                    watch_a_set = true;
                    break;
                }
                Some(value) if value == literal.polarity() => {
                    self.note_watch(literal.atom(), literal.polarity(), atoms);
                    self.clause.swap(0, index);
                    watch_a_set = true;
                    break;
                }
                Some(_) => {}
            }
        }
        if !watch_a_set {
            let zero_literal = unsafe { self.clause.get_unchecked(0) };
            self.note_watch(zero_literal.atom(), zero_literal.polarity(), atoms);
        }

        let mut watch_b_set = false;
        self.watch_ptr = 1;
        for index in 1..self.clause.len() {
            let literal = unsafe { self.clause.get_unchecked(index) };
            match atoms.value_of(literal.atom()) {
                None => {
                    self.watch_ptr = index;
                    self.note_watch(literal.atom(), literal.polarity(), atoms);
                    watch_b_set = true;
                    break;
                }
                Some(value) if value == literal.polarity() => {
                    self.watch_ptr = index;
                    self.note_watch(literal.atom(), literal.polarity(), atoms);
                    watch_b_set = true;
                    break;
                }
                Some(_) => {}
            }
        }

        if !watch_b_set {
            let ptr_literal = unsafe { self.clause.get_unchecked(self.watch_ptr) };
            self.note_watch(ptr_literal.atom(), ptr_literal.polarity(), atoms);
        }
    }

    /// Creates a watch tag and notes the given atom is now watched for being assigned the given value.
    ///
    /// # Safety
    /// A binary clause contains two literals, and so the use of get_unchecked is safe.
    pub fn note_watch(&self, atom: Atom, value: bool, atoms: &mut AtomDB) {
        match self.key {
            ClauseKey::Unit(_) => {
                panic!("attempting to interact with watches on a unit clause")
            }
            ClauseKey::Binary(_) => unsafe {
                let check_literal = if self.clause.get_unchecked(0).atom() == atom {
                    *self.clause.get_unchecked(1)
                } else {
                    *self.clause.get_unchecked(0)
                };

                atoms.add_watch_unchecked(atom, value, WatchTag::Binary(check_literal, self.key()));
            },
            ClauseKey::Original(_) | ClauseKey::Addition(_, _) => unsafe {
                atoms.add_watch_unchecked(atom, value, WatchTag::Clause(self.key()));
            },
        }
    }

    /// On the assumption that the given atom corresponds to a watched literal which as been assigned some value, updates the watched literals.
    ///
    /// As a guarantee of the above assumption would require inspection of both watched literals before any other action is taken, it is assumed.
    ///
    /// # Safety
    /// No checks on atom as index.
    #[inline(always)]
    #[allow(clippy::result_unit_err)]
    pub unsafe fn update_watch(
        &mut self,
        atom: Atom,
        atoms: &mut AtomDB,
    ) -> Result<WatchStatus, ()> {
        // assert!(self.clause.len() > 2);

        if self.clause.get_unchecked(0).atom() == atom {
            self.clause.swap(0, self.watch_ptr)
        }

        /*
        This loop could be split into two `for` loops around the current last index.
        This would avoid the need to check whether the search pointer is equal to where the last search pointer stopped.
        Still, it seems the single loop is easier to handle for the compiler.
         */
        let watch_ptr_cache = self.watch_ptr;
        let clause_length = self.clause.len();
        loop {
            self.watch_ptr += 1;
            if self.watch_ptr == clause_length {
                self.watch_ptr = 1 // skip 0
            }
            if self.watch_ptr == watch_ptr_cache {
                break Err(());
            }
            let literal = unsafe { self.clause.get_unchecked(self.watch_ptr) };
            match atoms.value_of(literal.atom()) {
                None => {
                    self.note_watch(literal.atom(), literal.polarity(), atoms);
                    break Ok(WatchStatus::None);
                }
                Some(value) if value == literal.polarity() => {
                    self.note_watch(literal.atom(), literal.polarity(), atoms);
                    break Ok(WatchStatus::Witness);
                }
                Some(_) => {}
            }
        }
    }
}
