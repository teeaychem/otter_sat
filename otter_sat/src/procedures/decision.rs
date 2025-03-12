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

Atoms may be selected by activity, and the [atom database](crate::db::atom) stores atoms without a value on a max value activity heap in order to support quick access to the most active atom without a value.
Though, as storing *only* without a value takes considerably more effort than *at least* those atoms without a value, it may take some work to find the relevant atom.

```rust,ignore
while let Some(atom) = self.atom_db.heap_pop_most_active() {
    if self.atom_db.value_of(atom as Atom).is_none() {
        return Some(atom);
    }
}
```

# Phase saving

If phase saving is enabled if a chosen atom was previously valued *v* the atom is again valued *v*.

```rust,ignore
let previous_value = self.atom_db.previous_value_of(chosen_atom);
CLiteral::new(chosen_atom, previous_value);
```

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
    /// Returns a result detailing the status of the decision or an error from attempting to enque the decision.
    ///
    /// ```rust, ignore
    /// match self.make_decision()? {
    ///     decision::Ok::Made => continue,
    ///     decision::Ok::Exhausted => break,
    /// }
    /// ```
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
                        let previous_value = self.atom_db.previous_value_of(chosen_atom);
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
    ///
    /// ```rust,ignore
    /// let atom = self.atom_without_value(MinimalPCG32::default())?;
    /// ```
    pub fn atom_without_value(&mut self, rng: &mut impl Rng) -> Option<Atom> {
        match rng.random_bool(self.config.random_decision_bias.value) {
            true => self.atom_db.valuation().unvalued_atoms().choose(rng),
            false => {
                while let Some(atom) = self.atom_db.heap_pop_most_active() {
                    if self.atom_db.value_of(atom as Atom).is_none() {
                        return Some(atom);
                    }
                }
                self.atom_db.valuation().unvalued_atoms().next()
            }
        }
    }

    /// Resets all decisions and consequences of those choises.
    ///
    /// In other words, backjumps to before any decision was made.
    /// Note, this does not clear any assumptions made.
    pub fn clear_decisions(&mut self) {
        self.state = ContextState::Input;
        self.backjump(self.atom_db.lowest_decision_level());
    }
}
