use rand::{seq::IteratorRandom, Rng};

use crate::{
    context::GenericContext,
    db::dbStatus,
    structures::{
        atom::Atom,
        literal::{abLiteral, Literal},
        valuation::Valuation,
    },
    types::err,
};

/// Possible 'Ok' results from choosing a truth value to assign an atom.
pub enum Ok {
    /// Some truth value was assigned to some atom.
    Made,
    /// All atoms had already been assigned truth values, so no choice could be made.
    Exhausted,
}

/// Methods related to making choices.
impl<R: rand::Rng + std::default::Default> GenericContext<R> {
    /// Makes a choice using rng to determine whether to make a random choice or to take the atom with the highest activity.
    ///
    /// Returns a result detailing the status of the choice or an error from attempting to enque the choice.
    ///
    /// ```rust, ignore
    /// match self.make_choice()? {
    ///     choice::Ok::Made => continue,
    ///     choice::Ok::Exhausted => break,
    /// }
    /// ```
    pub fn make_choice(&mut self) -> Result<Ok, err::Queue> {
        // Takes ownership of rng to satisfy the borrow checker.
        // Avoidable, at the cost of a less generic atom method.
        let mut rng = std::mem::take(&mut self.rng);
        let chosen_atom = self.atom_without_value(&mut rng);
        self.rng = rng;
        match chosen_atom {
            Some(choice_id) => {
                self.counters.total_choices += 1;

                let choice_literal = {
                    if self.config.switch.phase_saving {
                        let previous_value = self.atom_db.previous_value_of(choice_id);
                        abLiteral::fresh(choice_id, previous_value)
                    } else {
                        abLiteral::fresh(choice_id, self.rng.gen_bool(self.config.polarity_lean))
                    }
                };
                log::trace!("Choice {choice_literal}");
                self.literal_db.note_choice(choice_literal);
                self.q_literal(choice_literal)?;

                Ok(Ok::Made)
            }
            None => {
                self.status = dbStatus::Consistent;
                Ok(Ok::Exhausted)
            }
        }
    }

    /// Returns an atom which has no value on the current valuation, either by random choice or by most activity.
    ///
    /// ```rust,ignore
    /// let atom = self.atom_without_value(MinimalPCG32::default())?;
    /// ```
    pub fn atom_without_value(&mut self, rng: &mut impl Rng) -> Option<Atom> {
        match rng.gen_bool(self.config.random_choice_frequency) {
            true => self.atom_db.valuation().unvalued_atoms().choose(rng),
            false => {
                while let Some(index) = self.atom_db.heap_pop_most_active() {
                    if self.atom_db.value_of(index as Atom).is_none() {
                        return Some(index);
                    }
                }
                self.atom_db.valuation().unvalued_atoms().next()
            }
        }
    }

    /// Resets all choices and consequences of those choises.
    ///
    /// In other words, backjumps to before a choice was made.
    /// ```rust, ignore
    /// context.clear_choices();
    /// ```
    pub fn clear_choices(&mut self) {
        self.backjump(0);
    }
}
