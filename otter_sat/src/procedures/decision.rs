/*!
Methods for choosing the value of an atom.

# Overview

The core decision procedure is straightforward:
- Search through all atoms in the context for an atom which is not assigned a value, and assign either true or false.

```rust,ignore
self.atom_db.valuation().unvalued_atoms().next();
// Or…
self.atom_db.valuation().unvalued_atoms().choose(rng_source);
```

# Decisions as literals

Strictly a decision is to value some atom *a* with value *v*.
Still, it is convenient to represent such a decision as a literal with atom *a* and polarity *v*.
For example, a decision to value *p* with value *false* can be represented with the literal *-p*.

```rust,ignore
let atom = self.atom_db.valuation().unvalued_atoms().next()?;
let value = self.rng.random_bool(self.config.polarity_lean);
let decision_as_literal = CLiteral::new(atom, value);
```

# Heuristics

# Activity

Atoms are paired with an `activity` value.
Activity appoximates the relative degree to which the atom has been involved in deriving a conflict from BCP.
And, in particular, when a decision on some atom is required a decision on an atom with high activity may be (and in by default) preferred with the goal of quickly identifying whether the decision would lead to an unsatisfiable assignment.

For quick access to atoms with high activity values, atoms are stored on a custom max activity heap, which also supports tracking the activity of atoms not currently on the heap.

Likewise, clauses are paired with an activity value which appoximates the relative degree to which the clause has been involved in deriving a conflict from BCP.
And, when removing clauses from the database, clauses with low acitivty are removed ahead of clauses with a higher activity (though in the case of clauses, additional considerations factor into the ordering of clauses).

# Phase saving

If phase saving is enabled if a chosen atom was previously valued *v* the atom is again valued *v*.

Note: For efficiency an atom always has a 'previous' value, initialised randomly via [Config::polarity_lean](crate::config::Config::polarity_lean).

# Randomness

Use of activity, phase saving, or any other heuristic may be probabilistic, and likewise for the decision of atom and the decision of polarity.
*/

use rand::{Rng, seq::IteratorRandom};

use crate::{
    context::{ContextState, GenericContext},
    structures::{
        atom::Atom,
        literal::{CLiteral, Literal},
        valuation::Valuation,
    },
};

/// Possible 'Ok' results from choosing a truth value to assign an atom.
pub enum DecisionOk {
    /// Some truth value was assigned to some atom.
    Literal(CLiteral),

    /// All atoms had already been assigned truth values, so no decision could be made.
    Exhausted,
}

/// Methods related to making decisions.
impl<R: rand::Rng + std::default::Default> GenericContext<R> {
    /// Makes a decision using rng to determine whether to make a random decision or to take the atom with the highest activity.
    ///
    /// Returns a result detailing the status of the decision or an error from attempting to enqueue the decision.
    pub fn make_decision(&mut self) -> DecisionOk {
        // Takes ownership of rng to satisfy the borrow checker.
        // Avoidable, at the cost of a less generic atom method.
        let mut rng = std::mem::take(&mut self.rng);
        let chosen_atom = self.atom_without_value(&mut rng);
        self.rng = rng;
        match chosen_atom {
            Some(chosen_atom) => {
                self.counters.total_decisions += 1;

                let decision_literal = match self.config.phase_saving.value {
                    true => {
                        let previous_value = self.atom_cells.previous_value_of(chosen_atom);
                        CLiteral::new(chosen_atom, previous_value)
                    }
                    false => {
                        let random_value = self.rng.random_bool(self.config.polarity_lean.value);
                        CLiteral::new(chosen_atom, random_value)
                    }
                };
                log::trace!("Decision {decision_literal}");

                DecisionOk::Literal(decision_literal)
            }
            None => {
                self.state = ContextState::Satisfiable;
                DecisionOk::Exhausted
            }
        }
    }

    /// Returns an atom which has no value on the current valuation, either by random decision or by most activity.
    pub fn atom_without_value(&mut self, rng: &mut impl Rng) -> Option<Atom> {
        match rng.random_bool(self.config.random_decision_bias.value) {
            true => self.assignment().unvalued_atoms().choose(rng),
            false => {
                while let Some(atom) = self.atom_activity.pop_max().map(|idx| idx as Atom) {
                    if self.value_of(atom as Atom).is_none() {
                        return Some(atom);
                    }
                }
                self.assignment().unvalued_atoms().next()
            }
        }
    }

    /// Resets all decisions and consequences of those choices.
    ///
    /// In other words, backjumps to before any decision was made.
    /// Note, this does not clear any assumptions made.
    pub fn clear_decisions(&mut self) {
        self.state = ContextState::Input;
        self.backjump(self.trail.lowest_decision_level());
    }
}
