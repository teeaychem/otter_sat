/*!
A context method to aid boolean constraint propagation

See [GenericContext::bcp] for the relevant context method.

# Overview
Propagates an atom being assigned some value, given as a literal.

This is done by examining clauses watching the atom with the opposite polarity and updating the watches of the clause, if possible, queuing the consequence of the asserting clause, or identifying the clause conflicts with the current valuation.

# Complications

Use is made of [watchers_binary_unchecked](crate::db::atom::AtomDB::watchers_binary_unchecked) and [watchers_long_unchecked](crate::db::atom::AtomDB::watchers_long_unchecked) to obtain pointers to watch lists of interest.
A handful of issues are avoided by doing this:
1. A mutable borrow of the database for a watch list conflicting with an immutable borrow of the database to obtain the value of an atom.
2. A mutable borrow of the context conflicting with a mutable borrow to add a literal to the consequence queue.
3. A mutable borrow of the database in a call to update the watched literals in some clause.

(1) and (2) could be avoided by a more nuanced borrow checker, as these are separate structures, combined to ease reasoning about the library.
This is not the case for (3), as a watch list has been borrowed, and a call to [dbClause::update_watch](crate::db::clause::db_clause::dbClause::update_watch) may mutate watch lists.
Still, the *borrowed* watch list will not be mutated.
For, the literal bcp is being called on has been given some value, and the inspected list being for the atom with the opposite value.
And, the atom with the opposite value is not a [candidate](crate::db::clause::db_clause::dbClause) for updating a watch to as it:
- Has some value.
- Has a value which conflicts with the current valuation.

# Heuristics

Propagation happens in two steps, distinguished by clauses length:
- First, with respect to binary clauses.
- Second, with respect to long clauses.

This sequence is motivated by various considerations.
For example, binary clauses always have an lbd of at most 2, binary clauses do not require accessing the clause database and updating watches, etc.

# Example

bcp is a mutating method, and a typical application will match against the result of the mutation.
For example, a conflict may lead to conflict analysis and no conflict may lead to a decision being made.

```rust,ignore
match self.bcp(literal) {
    Err(err::BCP::Conflict(key)) => {
        if self.literal_db.decision_made() {
            let analysis_result = self.conflict_analysis(&clause_key)?;
            ...
        }
    }
    ...
    Ok => {
        match self.make_decision()? {
            ...
        }
    }
}
```
*/
use std::borrow::Borrow;

use crate::{
    context::GenericContext,
    db::{
        atom::{
            watch_db::{self},
            AtomValue,
        },
        consequence_q::{QPosition, QueueResult},
    },
    misc::log::targets::{self},
    structures::{
        consequence::{Assignment, AssignmentSource},
        literal::{CLiteral, Literal},
    },
    types::err::{self},
};

impl<R: rand::Rng + std::default::Default> GenericContext<R> {
    /// For documentation see [procedures::bcp](crate::procedures::bcp).
    ///
    /// # Soundness
    /// The implementation of BCP requires a key invariant to be upheld:
    /// <div class="warning">
    /// The literal at index 0 is a watched literal.
    /// </div>
    ///
    /// # Safety
    pub fn bcp(&mut self, literal: impl Borrow<CLiteral>) -> Result<(), err::BCPError> {
        let literal = literal.borrow();
        let decision_level = self.literal_db.current_level();

        /*
        # Safety

        The binary and long blocks are both wrapped in unsafe to keep specific unsafe instances simple.

        Use of unsafe operations is motivated by two isses:

        - When traversing through a list, watches may be dropped.
          For this an index to the current element is used, and the element retreived when needed.
          As custom checks are made to ensure this traveral works, accessing the element is unchecked.

        - When updating a watch the consequence queue may be updated, requiring a split borrow of a context.
          As the consequence queue is not examined until after the current instance of BCP is complete, this is safe.

        Note, further, that even if BCP were applied aggressively, with each propagation immediately calling BCP, the implementation would remain safe.
        For, the literal under consideration has been set, and as such is not a candidate for an updated watch.
        */

        // Binary clause block.
        unsafe {
            // Note, this does not require updating watches.
            let binary_list = self.atom_db.watchers_binary_unchecked(literal);

            for element in &*binary_list {
                let check = element.literal;
                let key = element.key;

                match self.atom_db.value_of(check.atom()) {
                    None => match self.value_and_queue(check, QPosition::Back, decision_level) {
                        AtomValue::NotSet => {
                            let consequence = Assignment::from(check, AssignmentSource::BCP(key));
                            self.record_consequence(consequence);
                        }

                        AtomValue::Same => {}

                        AtomValue::Different => return Err(err::BCPError::Conflict(key)),
                    },

                    Some(value) if check.polarity() != value => {
                        log::trace!(target: targets::PROPAGATION, "Consequence of {key} and {literal} is contradiction.");
                        return Err(err::BCPError::Conflict(key));
                    }

                    Some(_) => {
                        log::trace!(target: targets::PROPAGATION, "Repeat implication of {key} {literal}.");
                        // a repeat implication, as this is binary
                    }
                }
            }
        }

        // Long clause block.
        unsafe {
            let long_list = &mut *self.atom_db.watchers_long_unchecked(literal);

            let mut index = 0;
            let mut length = long_list.len();

            'long_loop: while index < length {
                let key = long_list.get_unchecked(index).key;

                let db_clause = match self.clause_db.get_mut(&key) {
                    Ok(stored) => stored,
                    Err(_) => {
                        length -= 1;
                        long_list.swap(index, length);
                        continue 'long_loop;
                    }
                };

                match db_clause.update_watch(literal.atom(), &mut self.atom_db) {
                    Ok(watch_db::WatchStatus::Witness) | Ok(watch_db::WatchStatus::None) => {
                        length -= 1;
                        long_list.swap(index, length);
                        continue 'long_loop;
                    }

                    Ok(watch_db::WatchStatus::Conflict) => {
                        log::error!(target: targets::PROPAGATION, "Conflict from updating watch during propagation.");
                        long_list.split_off(length);
                        return Err(err::BCPError::CorruptWatch);
                    }

                    Err(()) => {
                        // After the call to update_watch, any atom without a value will be in position 0.
                        let watch = *db_clause.get_unchecked(0);

                        match self.atom_db.value_of(watch.atom()) {
                            Some(value) if watch.polarity() != value => {
                                self.clause_db.note_use(key);

                                long_list.split_off(length);
                                return Err(err::BCPError::Conflict(key));
                            }

                            None => {
                                self.clause_db.note_use(key);

                                match self.value_and_queue(watch, QPosition::Back, decision_level) {
                                    AtomValue::NotSet => {
                                        let consequence =
                                            Assignment::from(watch, AssignmentSource::BCP(key));
                                        self.record_consequence(consequence);
                                    }

                                    AtomValue::Same => {}

                                    AtomValue::Different => {
                                        long_list.split_off(length);
                                        return Err(err::BCPError::Conflict(key));
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

            long_list.split_off(length);
        }
        Ok(())
    }
}
