/*!
(The internal representation of) an atom (aka. a 'variable').

Broadly, atoms are things with a name to which assigning a (boolean) value (true or false) is of interset.
- 'Internal' atoms are used internal to a context.
- 'External' atoms are used to used during external interaction with a context, e.g. when providing a formula as input or reading the value of an atom. \
     External atoms are a string of non-whitespace characters that which does not being with '-' (a minus sign). \
     Examples: `p`, `atom_one`, `96`, `0`.

Each (*internal+) atom is a u32 *u* such that either:
- *u* is 0, or:
- *u - 1* is an atom.

```rust
# use otter_sat::structures::atom::Atom;
let m = 97;
let atoms = (0..m).collect::<Vec<Atom>>();

let atom = 97;
```

That the atoms are [0..*m*) for some *m*.

This representation allows atoms to be used as the indicies of a structure, e.g. `exteranal_string[a]` without taking too much space.
Revising the representation to any unsigned integer is possible.

# Notes
- The external representation of an atom is stored in the atom database.
- In the SAT literature these are often called 'variables' while in the logic literature these are often called 'atoms'.
*/

/// An atom, aka. a 'variable'.
pub type Atom = u32;

/// The atom `0` is fixed internally with a value of true.
static TOP_ATOM: Atom = 0;

/// The maximum instance of an atom.
#[cfg(feature = "boolean")]
pub const ATOM_MAX: Atom = Atom::MAX;

/// The maximum instance of an atom.
#[cfg(not(feature = "boolean"))]
pub const ATOM_MAX: Atom = i32::MAX.unsigned_abs();
