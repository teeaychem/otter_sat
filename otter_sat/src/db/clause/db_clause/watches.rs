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
        valuation::{vValuation, Valuation},
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
    pub unsafe fn get_watch_a(&self) -> &abLiteral {
        self.get_unchecked(0)
    }

    pub unsafe fn get_watch_b(&self) -> &abLiteral {
        self.get_unchecked(self.watch_ptr)
    }

    /// Initialises watches.
    /*
    # Note
    In order to avoid redundant literal lookup, watch candidates are noted when found.
    Still, as a watch candidate may *fail* to be found it is important to check and note the watch regardless.

     Failure for a candidate for watch A to be found implies a candidate for watch B.
     Still, this is not encoded, as failure for watch A is very unlikely.
     */
    pub fn initialise_watches(&mut self, atom_db: &mut AtomDB, valuation: Option<&vValuation>) {
        // As watches require two or more literals, and watch_ptr must be within the bounds of the vector, use of get_unchecked on index zero and watch_ptr is safe.
        let mut watch_a_set = false;

        for (index, literal) in self.clause.iter().enumerate() {
            let index_value = match valuation {
                Some(v) => unsafe { v.value_of_unchecked(literal.atom()) },
                None => unsafe { atom_db.valuation().value_of_unchecked(literal.atom()) },
            };

            match index_value {
                None => {
                    self.note_watch(literal.atom(), literal.polarity(), atom_db);
                    self.clause.swap(0, index);
                    watch_a_set = true;
                    break;
                }
                Some(value) if value == literal.polarity() => {
                    self.note_watch(literal.atom(), literal.polarity(), atom_db);
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
            self.note_watch(zero_literal.atom(), zero_literal.polarity(), atom_db);
        }

        // For the other watch literal an unvalued or satisfied literal is chosen over an unsatisfied literal.
        // Still, if there is no other choice, the pointer will rest on some unsatisfied literal with the highest decision level.
        let mut watch_b_set = false;
        self.watch_ptr = 1;
        let mut decision_level_b = unsafe {
            let literal = self.clause.get_unchecked(self.watch_ptr);
            let maybe_decision_level = atom_db.decision_index_of(literal.atom());
            maybe_decision_level.unwrap_or(0)
        };

        for index in 1..self.clause.len() {
            let literal = unsafe { self.clause.get_unchecked(index) };

            let atom_value = match valuation {
                Some(v) => unsafe { v.value_of_unchecked(literal.atom()) },
                None => unsafe { atom_db.valuation().value_of_unchecked(literal.atom()) },
            };

            match atom_value {
                None => {
                    self.watch_ptr = index;
                    self.note_watch(literal.atom(), literal.polarity(), atom_db);
                    watch_b_set = true;
                    break;
                }
                Some(value) if value == literal.polarity() => {
                    self.watch_ptr = index;
                    self.note_watch(literal.atom(), literal.polarity(), atom_db);
                    watch_b_set = true;
                    break;
                }
                Some(_) => {
                    let decision_level =
                        unsafe { atom_db.decision_index_of(literal.atom()).unwrap_unchecked() };
                    if decision_level > decision_level_b {
                        self.watch_ptr = index;
                        decision_level_b = decision_level;
                    }
                }
            }
        }

        if !watch_b_set {
            let ptr_literal = unsafe { self.clause.get_unchecked(self.watch_ptr) };
            self.note_watch(ptr_literal.atom(), ptr_literal.polarity(), atom_db);
        }
    }

    /// Creates a watch tag and notes the given atom is now watched for being assigned the given value.
    ///
    /// # Safety
    /// A binary clause contains two literals, and so the use of get_unchecked is safe.
    pub fn note_watch(&self, atom: Atom, value: bool, atom_db: &mut AtomDB) {
        match self.key {
            ClauseKey::Unit(_) => {
                panic!("!")
            }
            ClauseKey::Binary(_) => unsafe {
                // For binary watches, the other watched literal is included in the watch tag.
                let check_literal = if self.clause.get_unchecked(0).atom() == atom {
                    *self.clause.get_unchecked(1)
                } else {
                    *self.clause.get_unchecked(0)
                };

                atom_db.add_watch_unchecked(
                    atom,
                    value,
                    WatchTag::Binary(check_literal, self.key()),
                );
            },
            ClauseKey::Original(_) | ClauseKey::Addition(_, _) => unsafe {
                atom_db.add_watch_unchecked(atom, value, WatchTag::Clause(self.key()));
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
        atom_db: &mut AtomDB,
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
            match atom_db.value_of(literal.atom()) {
                None => {
                    self.note_watch(literal.atom(), literal.polarity(), atom_db);
                    break Ok(WatchStatus::None);
                }
                Some(value) if value == literal.polarity() => {
                    self.note_watch(literal.atom(), literal.polarity(), atom_db);
                    break Ok(WatchStatus::Witness);
                }
                Some(_) => {}
            }
        }
    }
}
