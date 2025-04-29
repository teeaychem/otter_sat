# Overview

Otter… other… odder… otter_sat is a library for determining the satisfiability of boolean formulas written in conjunctive normal form.

otter_sat uses a variety of techniques from the literature on conflict-driven clause-learning solving, and with support for incremental solves.
In particular, otter_sat supports the IPASIR API (see below).

otter_sat is developed to help researchers, developers, or anyone curious, to investigate satisfiability solvers, whether as a novice or through implementing novel ideas.

- For documentation, see [https://docs.rs/otter_sat](https://docs.rs/otter_sat).
- For an associated cli application and tools to verify (parts of) the library see [https://github.com/teeaychem/otter_sat](https://github.com/teeaychem/otter_sat).

Some guiding principles of the library are:
- Modularity.
- Documentation, of both theory and implementation.
- Verification (of core parts of the library through the production of FRAT proofs and external proof checkers).
- Simple efficiency.

# Use

- The documentation contains various examples, and a handful of example files are included in the crate.
  Example can be run with `cargo run --example <EXAMPLE>`.
  For example, `cargo run --example sudoku`.
  Though, complex examples are much faster using a release (or similar) profile.
  For example, `cargo run --example nonograms --profile release`.
- [otter_tests](https://github.com/teeaychem/otter_sat/tree/main/otter_tests) is a crate to test the library against known problems, and to verify produced FRAT proofs and as such contains a variety of illustrative functions to help achieve these tasks.
- [otter_cli](https://github.com/teeaychem/otter_sat/tree/main/otter_cli) is a cli frontend to the library, which supports DIMACS encoded CNFs.

# CLI Binary

otter_sat includes a (simple) CLI.
Files are contained in the `cli` directory, and the binary is built as `otter_cli`.

The source of the binary functions as examples of how to identify unsatisfiable cores, write FRAT proofs, and set callbacks.

# IPASIR

C bindings for the IPASIR API are included.
To use these bindings otter_sat should be compiled in a suitable way, e.g. as a cdylib.
For example, via:

```sh
cargo rustc --crate-type=cdylib
```

# Caveats

The solver is developed to help those curious about sat, and that includes me.
In particular, the core solver is the implementation of theory, without peeks at other implementations.
So, some parts are likely to be idiosyncratic and perhaps quite inefficient.
As development continues, insights from other solvers will (hopefully) be incorporated.

Documentation is for the moment quite irregular.
Most structs, functions, etc. have *some* documentation, and *some* structs, functions, etc. have (perhaps) useful documentation.

The design of the solver is only 'mostly' stable.
Many too-be-implemented features (bound variable elimination, vivification, etc.) would be additive.
Still, taming idiosyncrasies, support for SMT solving, and interest in very large problems may require more fundamental revisions.

# Examples

- Find (a count of) all valuations of some collection of atoms

``` rust
// The context in which a solve takes place.
let mut context: Context = Context::from_config(Config::default());

// Atoms will be represented by characters of some string.
let characters = "model".chars().collect::<Vec<_>>();
let mut atom_count: u32 = 0;

// Each call to fresh_atom expands the context to include a fresh (new) atom.
// Atoms form a contiguous range from 1 to some limit.
for _character in &characters {
    match context.fresh_atom() {
        Ok(_) => atom_count += 1,
        Err(_) => {
            panic!("Atom limit exhausted.")
        }
    }
}

let mut model_count = 0;

while let Ok(Report::Satisfiable) = context.solve() {
    model_count += 1;

    let mut valuation_representation = String::new();

    // To exclude the current valuation, the negation of the current valuation is added as a clause.
    // As valuations are conjunctions and clauses disjunctions, this may be done by negating each literal.
    let mut exclusion_clause = Vec::new();

    // The context provides an iterator over (atom, value) pairs.
    // Though every non-constant atom has a value in this model, this avoids handling the no value option.
    for (atom, value) in context.assignment().atom_valued_pairs() {
        // As atoms begin at 1, a step back is required to find the appropriate character.
        match value {
            true => valuation_representation.push(' '),
            false => valuation_representation.push('-'),
        }
        valuation_representation.push(characters[(atom as usize) - 1]);
        valuation_representation.push(' ');

        exclusion_clause.push(CLiteral::new(atom as Atom, !value));
    }

    valuation_representation.pop();

    // After a solve, the context is refreshed to clear any decisions made.
    // Learnt clauses remain, though any assumptions made are also removed.
    context.refresh();

    match context.add_clause(exclusion_clause) {
        Ok(_) => {}
        Err(_) => break,
    };
}

assert_eq!(model_count, 2_usize.pow(atom_count));
```

- Parse and solve a DIMACS formula

``` rust
let mut ctx = Context::from_config(Config::default());

let mut dimacs = vec![];
let _ = dimacs.write(b"
 1  2 0
-1  2 0
-1 -2 0
 1 -2 0
");

ctx.read_dimacs(dimacs.as_slice());
ctx.solve();
assert_eq!(ctx.report(), report::Solve::Unsatisfiable);
```
