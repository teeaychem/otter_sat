use rand::{seq::IteratorRandom, Rng};

use crate::{
    context::Context,
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

impl Context {
    pub fn make_choice(&mut self) -> Result<Ok, err::Queue> {
        match self.get_unassigned() {
            Some(choice_id) => {
                self.counters.choices += 1;

                let choice_literal = {
                    if self.config.switch.phase_saving {
                        let previous_value = self.atom_db.previous_value_of(choice_id);
                        abLiteral::fresh(choice_id, previous_value)
                    } else {
                        abLiteral::fresh(
                            choice_id,
                            self.counters.rng.gen_bool(self.config.polarity_lean),
                        )
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

    pub fn get_unassigned(&mut self) -> Option<Atom> {
        match self
            .counters
            .rng
            .gen_bool(self.config.random_choice_frequency)
        {
            true => self
                .atom_db
                .valuation()
                .unvalued_atoms()
                .choose(&mut self.counters.rng),
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

    pub fn clear_choices(&mut self) {
        self.backjump(0);
    }
}
