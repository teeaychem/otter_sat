use crate::{
    atom_cells::AtomCells,
    db::{
        keys::ClauseKey,
        watches::{
            Watches,
            watch_db::{BinaryWatch, LongWatch, WatchStatus},
        },
    },
    structures::{
        atom::Atom,
        literal::{CLiteral, Literal},
        valuation::Valuation,
    },
};

use super::dbClause;

/// Methods for watched literals of a clause.
///
/// For more details on watched literals see documentation of the [watch_db](crate::db::atom::watch_db) structure.
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
/// So, there are two key invariants to keep maintained:
/// <div class="warning">
/// The first literal and the literal at the index of watch_ptr are watch candidates, if any exist.
/// </div>
/// <div class="warning">
/// After any update to the watched literals, the first literal has no value on the current valuation, if possible.
/// </div>
///
impl dbClause {
    /// Returns the first watched literal (of two).
    pub fn get_watch_a(&self) -> &CLiteral {
        // # Safety
        // db clauses have at least two literals
        unsafe { self.get_unchecked(0) }
    }

    /// Returns the second watched literal (of two).
    pub fn get_watch_b(&self) -> &CLiteral {
        // # Safety
        // db clauses have at least two literals
        unsafe { self.get_unchecked(self.watch_ptr) }
    }

    /// Initialises watches with respect to the given valuation, or otherwise to the current valuation of the context.
    /*
    # Note
    In order to avoid redundant literal lookup, watch candidates are noted when found.
    Still, as a watch candidate may *fail* to be found it is important to check and note the watch regardless.

     Failure for a candidate for watch A to be found implies a candidate for watch B.
     Still, this is not encoded, as failure for watch A is very unlikely.
     */
    pub fn initialise_watches(&mut self, cells: &mut AtomCells, watches: &mut Watches) {
        // As watches require two or more literals, and watch_ptr must be within the bounds of the vector, use of get_unchecked on index zero and watch_ptr is safe.
        let mut watch_a_set = false;

        for (index, literal) in self.clause.iter().enumerate() {
            let index_value = unsafe { cells.value_of_unchecked(literal.atom()) };

            match index_value {
                None => {
                    self.note_watch(literal, watches);
                    self.clause.swap(0, index);
                    watch_a_set = true;
                    break;
                }

                Some(value) if value == literal.polarity() => {
                    self.note_watch(literal, watches);
                    self.clause.swap(0, index);
                    watch_a_set = true;
                    break;
                }

                Some(_) => {}
            }
        }
        if !watch_a_set {
            // May fail if an appropriate backjump has not been made before adding a clause.
            let zero_literal = unsafe { self.clause.get_unchecked(0) };
            self.note_watch(zero_literal, watches);
        }

        // For the other watch literal an unvalued or satisfied literal is chosen over an unsatisfied literal.
        // Still, if there is no other choice, the pointer will rest on some unsatisfied literal with the highest decision level.
        let mut watch_b_set = false;
        self.watch_ptr = 1;

        let mut level_b = {
            // # Safety: The watch_ptr always points to some literal in the clause
            let literal = unsafe { self.clause.get_unchecked(self.watch_ptr) };
            cells.level(literal.atom())
        };

        for index in 1..self.clause.len() {
            let literal = unsafe { self.clause.get_unchecked(index) };

            let atom_value = unsafe { cells.value_of_unchecked(literal.atom()) };
            match atom_value {
                None => {
                    self.watch_ptr = index;
                    self.note_watch(literal, watches);
                    watch_b_set = true;
                    break;
                }

                Some(value) if value == literal.polarity() => {
                    self.watch_ptr = index;
                    self.note_watch(literal, watches);
                    watch_b_set = true;
                    break;
                }

                Some(_) => {
                    // Safety: The clause has a value, which must have been given at some level.
                    let decision_level = unsafe { cells.level(literal.atom()).unwrap_unchecked() };

                    if level_b.is_none_or(|l| decision_level > l) {
                        self.watch_ptr = index;
                        level_b = Some(decision_level);
                    }
                }
            }
        }

        if !watch_b_set {
            // # Safety: The watch_ptr always points to some literal in the clause
            let ptr_literal = unsafe { self.clause.get_unchecked(self.watch_ptr) };
            self.note_watch(ptr_literal, watches);
        }
    }

    /// Creates a watch tag and notes the given atom is now watched for being assigned the given value.
    ///
    /// # Safety
    /// A binary clause contains two literals, and so the use of get_unchecked is safe.
    pub fn note_watch(&self, literal: &CLiteral, watch_db: &mut Watches) {
        match self.key {
            ClauseKey::OriginalUnit(_) | ClauseKey::AdditionUnit(_) => {
                panic!("! Attempt to note watches on a unit clause")
            }

            ClauseKey::OriginalBinary(_) | ClauseKey::AdditionBinary(_) => unsafe {
                // For binary watches, the other watched literal is included in the watch tag.
                let check_literal = if self.clause.get_unchecked(0).atom() == literal.atom() {
                    *self.clause.get_unchecked(1)
                } else {
                    *self.clause.get_unchecked(0)
                };

                watch_db
                    .watch_binary_unchecked(literal, BinaryWatch::new(check_literal, *self.key()));
            },

            ClauseKey::Original(_) | ClauseKey::Addition(_, _) => {
                watch_db.watch_long_unchecked(literal, LongWatch::new(*self.key()))
            }
        }
    }

    /// Updates the watched literals, given an atom whose value has been set.
    #[allow(clippy::result_unit_err)]
    pub fn update_watch(
        &mut self,
        atom: Atom,
        valuation: &impl Valuation,
        watches: &mut Watches,
    ) -> Result<WatchStatus, ()> {
        let watch_ptr_cache = self.watch_ptr;
        let clause_length = self.clause.len();

        // # Safety
        // The procedure makes unchecked access to literals in the clause.
        // Either direct to the initial clause, or indirect via watch_ptr.
        // Asserting watch_ptr < clause length ensures each use of watch_ptr is safe.
        // And, as watch_ptr is unsigned, this also ensure direct access to the first literal is safe.
        // assert!(watch_ptr_cache < clause_length);
        if self.get_watch_a().atom() == atom {
            self.clause.swap(0, self.watch_ptr)
        }

        /*
        This loop could be split into two `for` loops around the current last index.
        This would avoid the need to check whether the search pointer is equal to where the last search pointer stopped.
        Still, it seems the single loop is easier to handle for the compiler.
         */

        loop {
            let literal = unsafe { self.clause.get_unchecked(self.watch_ptr) };
            match unsafe { valuation.value_of_unchecked(literal.atom()) } {
                None => {
                    self.note_watch(literal, watches);
                    break Ok(WatchStatus::None);
                }

                Some(value) if value == literal.polarity() => {
                    self.note_watch(literal, watches);
                    break Ok(WatchStatus::Witness);
                }

                Some(_) => {}
            }

            self.watch_ptr += 1;

            // skip index 0, which contains the other watched literal
            if self.watch_ptr == clause_length {
                self.watch_ptr = 1
            }

            // If no update was made
            if self.watch_ptr == watch_ptr_cache {
                break Err(());
            }
        }
    }
}
