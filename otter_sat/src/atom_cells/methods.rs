/*!
A structure derive the resolution of some collection of clauses with stopping points.

Resolution allows the derivation of a clause from a collection of clauses.

- The *resolution* of two formulas φ ∨ *p* and ψ ∨ *-p* is the formula φ ∨ ψ.
  + Here:
    - φ and ψ stand for arbitrary disjunctions, such as *q ∨ r ∨ s* and *t*, etc.
    - *p* is called the 'pivot' for the instance resolution.
      More generally:
      * A *pivot* for a pair of clauses *c₁* and *c₂* is some literal *l* such that *l* is in *c₁* and -*l* is in *c₂*.
        - For example, *q* is a pivot for  *p ∨ -q* and *p ∨ q ∨ r*, as *-q* is in the first and *q* in the second.
          Similarly, there are two pivots in the pair of clauses *p ∨ -q* and *-p ∨ q*.

Resolution is defined for a pair of formulas, but may be chained indefinitely so long as some pivot is present.
For example, given *p ∨ -q ∨ -r* and *-p*, resolution can be used to derive *-q ∨ -r* and in turn the clause *r ∨ s* can be used to derive *-q ∨ s*.

Further, it is often useful to stop resolution when a clause becomes asserting on some valuation.
That is, when all but one literal conflicts with the valuation, as then the non-conflicting literal must hold on the valuation.

The structure here allows for an arbitrary chain of resolution instances with stopping points by:
- Setting up a vector containing cells for all atoms that may be relevant to the resolution chain.
- Updating the contents of each cell to indicate whether that atom is part of the derived clause, or has been used as a pivot.
- While, keeping track of which cells used in resolution conflict with the valuation.

In addition, the structure has been extended to support self-subsumption of clauses and clause minimization.


Note, at present, the structure creates a cell for each atom in the context.
This allows for a simple implementation, but is likely inefficient for a large collection of atoms.
Improvement could be made by temporarily mapping relevant atoms to a temporary sub-language derived from the clauses which are candidates for resolution (so long as this is a finite collection…)
*/

use std::collections::HashSet;

use crate::{
    atom_cells::{
        AtomCells,
        cell::{AtomCell, ResolutionFlag},
    },
    structures::{atom::Atom, consequence::AssignmentSource},
};

impl AtomCells {
    pub(crate) fn new() -> Self {
        Self {
            valueless_count: 0,
            clause_length: 0,
            premises: HashSet::default(),
            cells: Vec::default(),
            merged_atoms: Vec::default(),
            callback_premises: None,
            recursive_minimization_todo: Vec::default(),
            cached_removable_status_atoms: Vec::default(),
        }
    }

    pub fn refresh(&mut self) {
        self.valueless_count = 0;
        self.clause_length = 0;
        self.premises.clear();
    }

    pub fn grow_to_include(&mut self, atom: Atom) {
        if self.cells.len() <= atom as usize {
            self.cells.resize(atom as usize + 1, AtomCell::default());
        }
    }

    pub fn set_valuation(&mut self, atom: Atom, value: Option<bool>, source: AssignmentSource) {
        let cell = self.get_cell_mut(atom);
        cell.value = value;
        cell.source = source;
        cell.resolution_flag = ResolutionFlag::Valuation;
    }

    pub fn mark_backjump(&mut self, atom: Atom) {
        let cell = self.get_cell_mut(atom);
        cell.resolution_flag = ResolutionFlag::Backjump;
    }

    /// Sets an atom to have no valuation in the resolution buffer.
    ///
    /// Useful to initialise the resolution buffer with the current valuation and then to 'roll it back' to the previous valuation.
    pub fn clear_value(&mut self, atom: Atom) {
        let cell = self.get_cell_mut(atom);
        cell.value = None;
        cell.resolution_flag = ResolutionFlag::Valuation;
    }
}
