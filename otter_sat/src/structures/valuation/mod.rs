/*!
A (partial) function from atoms to truth values.

If all atoms are assigned a value the valuation is 'full', otherwise the valuation is 'partial'.

The canonical representation of a valuation as a vector of optional booleans, where:
- The zero index (first) element is true, interpreted as some arbitrary tautology.
- Each non-zero index of the vector is interpreted as an atom, though most interaction is through the valuation trait.

 In other words, the canonical representation of a valuation 𝐯 is a vector *v* whose length is the number of atoms in the context such that:[^pedantic]
 -  *v*\[a\] = Some(true) *if any only if* 𝐯(𝐚) = true.
 -  *v*\[a\] = Some(false) *if any only if* 𝐯(𝐚) = false.
 -  *v*\[a\] = None *if any only if* 𝐯(𝐚) is undefined.

The trait is implemented for anything which can be dereferenced to a slice of optional booleans.

```rust
# use otter_sat::structures::atom;
# use otter_sat::structures::valuation::Valuation;
let atoms =     vec![0,          1,    2,          3];
let valuation = vec![Some(true), None, Some(true), None];

assert_eq!(unsafe { valuation.value_of_unchecked(1) }, None);
assert_eq!(valuation.unvalued_atoms().count(), 2);
```

Throughout the library the unsafe `value_of_unchecked` is preferred over the safe `value_of`. \
This is because the implementation on vectors 'only' guarantees *memory* safety, while use requires the stronger guarantee that the (optional) value atom of interest is mapped to the index of the atom in the valuation, and with this an additional check that the atom really is there is redundant.

[^pedantic]: Where 'a' is the internal representation of some atom whose external representation is '𝐚'.

# Falsum

The first element of a valuation is designated to be falsum as atoms are positive integers.
And, as the atoms in a solve are a contiguous slice of positive integers starting from 1 -- [1..] -- the value of atom *i* may be identified with the contents of the *i*th index of a vector.

True is used, in particular, as it allows the literal representation of a valuation to be interpreted as a conjunction.

# Soundness

The valuation trait is implemented for any structure which can be dereferenced to a slice of optional booleans.
And, as the value of an atom is determined by using the atom as an index on the dereferenced structure, there is no structural guarantee that the returned value is for the atom.

In other words, the following is possible, unsound, and by (design/luck) fails:

```rust,should_panic
# use otter_sat::structures::atom;
# use otter_sat::structures::valuation::Valuation;
let atoms =     vec![0,          1,    2,          3];
let valuation = vec![Some(true), None, Some(true), None];

let sub_valuation = valuation[1..].iter().copied().collect::<Vec<_>>();
assert_eq!(sub_valuation.value_of(1), Some(None));
```
*/

mod slice_impl;

use super::atom::Atom;

/// The canonical representation of a valuation.
#[allow(non_camel_case_types)]
pub type CValuation = Vec<Option<bool>>;

/// A valuation is something which stores some value of a atom and/or perhaps the information that the atom has no value.
pub trait Valuation {
    /// Some value of a atom under the valuation, or otherwise nothing.
    fn value_of(&self, atom: Atom) -> Option<Option<bool>>;

    /// Some value of a atom under the valuation, or otherwise nothing.
    /// # Safety
    /// Implementations are not required to check the atom is part of the valuation.
    unsafe fn value_of_unchecked(&self, atom: Atom) -> Option<bool>;

    /// An iterator over the values of a atoms in the valuation, in strict, contiguous, atom order.
    /// I.e. the first element is the atom '1' and then *n*th element is atom *n*.
    fn values(&self) -> impl Iterator<Item = Option<bool>>;

    /// An iterator through all (Atom, Value) pairs (excluding top).
    fn atom_value_pairs(&self) -> impl Iterator<Item = (Atom, Option<bool>)>;

    /// An iterator through all (Atom, Value) pairs for such that the atom has some value (excluding top).
    fn atom_valued_pairs(&self) -> impl Iterator<Item = (Atom, bool)>;

    /// An iterator through atoms which have some value.
    fn valued_atoms(&self) -> impl Iterator<Item = Atom>;

    /// An iterator through atoms which do not have some value.
    fn unvalued_atoms(&self) -> impl Iterator<Item = Atom>;

    /// The canonical representation of a valuation as a canonical valuation ([CValuation]).
    fn canonical(&self) -> CValuation;

    /// Ensures the first element of the valuation is false.
    fn true_check(&self) -> bool;

    /// Sets the value of the given atom to `None`.
    ///
    /// # Safety
    /// Implementations are not required to check the atom is part of the valuation.
    unsafe fn clear_value_of(&mut self, atom: Atom);

    /// A count of all the atoms in the valuation (including top).
    fn atom_count(&self) -> usize;
}
