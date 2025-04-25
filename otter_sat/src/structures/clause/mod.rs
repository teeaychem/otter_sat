/*!
Clauses, aka. a collection of literals, interpreted as the disjunction of those literals.

The canonical representation of a clause is as a vector of literals.

```rust
# use otter_sat::structures::literal::{CLiteral, Literal};
# use otter_sat::structures::clause::Clause;
let clause = vec![CLiteral::new(23, true),
                  CLiteral::new(41, false),
                  CLiteral::new(3,  false),
                  CLiteral::new(15, true),
                  CLiteral::new(4,  false)];

assert_eq!(clause.size(), 5);

let mut some_valuation = vec![Some(true); 42];

some_valuation[23] = Some(false);
some_valuation[15] = Some(false);
assert!(clause.asserts(&some_valuation).cmp(&None).is_eq());

some_valuation[41] = None;
assert!(clause.asserts(&some_valuation).cmp(&Some(CLiteral::new(41, false))).is_eq());
```

- The empty clause is always false (never true).
- Single literals are identified with the clause containing that literal (aka. a 'unit' clause --- where the 'unit' is the literal).
*/

mod db_clause;
mod kind;
mod literal;

mod ab_clause;
pub use ab_clause::ABClause;

mod int_clause;
pub use int_clause::IntClause;

mod source;
pub use source::ClauseSource;

pub use kind::ClauseKind;

use crate::{
    atom_cells::AtomCells,
    config::LBD,
    structures::{atom::Atom, literal::CLiteral, valuation::Valuation},
};

#[cfg(feature = "boolean")]
/// The canonical implementation of a clause.
pub type CClause = ABClause;

#[cfg(not(feature = "boolean"))]
/// The canonical implementation of a clause.
pub type CClause = IntClause;

/// The clause trait.
pub trait Clause {
    /// A string of the clause in DIMACS form, with the terminating `0` as optional.
    fn as_dimacs(&self, zero: bool) -> String;

    /// The literal asserted by the clause on a given valuation, if one such literal exists. \
    /// In detail, returns:
    /// - Some(*l*), if *l* has no value on the given valuation and for every other literal *l'* in the clause the polarity of *l'* conflicts with the value of the atom of *l'*.
    /// - None, otherwise.
    fn asserts<V: Valuation>(&self, val: &V) -> Option<CLiteral>;

    /// The Literal Block Distance of the clause.
    /// That is, the number of (distinct) decisions which influence the value of atoms in the clause.
    fn lbd(&self, cells: &AtomCells) -> LBD;

    /// An iterator over all literals in the clause, order is not guaranteed.
    fn literals(&self) -> impl Iterator<Item = CLiteral>;

    /// The number of literals in the clause.
    fn size(&self) -> usize;

    /// An iterator over all atoms in the clause, order is not guaranteed.
    fn atoms(&self) -> impl Iterator<Item = Atom>;

    /// The clause in its canonical form.
    fn canonical(self) -> CClause;

    /// Returns whether the clause is unsatisfiable on the given valuation
    fn unsatisfiable_on(&self, valuation: &impl Valuation) -> bool;

    /// Returns the literal at index 'index', if the clause contains at least `index` literals.
    fn literal_at(&self, index: usize) -> Option<CLiteral>;

    /// Returns the literal at `index`, without checking whether the clause contains at least `index` literals.
    ///
    /// # Safety
    /// Well-defined only if the clause contains at least `index` literals.
    unsafe fn literal_at_unchecked(&self, index: usize) -> CLiteral;

    /// Returns the atom at index 'index', if the clause contains at least `index` atoms.
    fn atom_at(&self, index: usize) -> Option<Atom>;

    /// Returns the atom at `index`, without checking whether the clause contains at least `index` atoms.
    ///
    /// # Safety
    /// Well-defined only if the clause contains at least `index` atoms.
    unsafe fn atom_at_unchecked(&self, index: usize) -> Atom;
}
