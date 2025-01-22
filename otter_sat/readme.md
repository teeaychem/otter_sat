# Overview

Otter… other… odder… otter_sat is a library for determining the satisfiability of boolean formulas written in conjunctive normal form.

otter_sat uses a variety of techniques from the literature on conflict-driven clause-learning solving, and with support for incremental solves.

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

## IPASIR

C bindings are included.

- Compile the crate as a dynamic library

```sh
cargo rustc --crate-type=cdylib
```

Generate a header file using [cbindgen](https://github.com/mozilla/cbindgen)

```sh
cbindgen --config cbindgen.toml --crate otter_sat --output otter_ipasir.h
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
Still, taming idiosyncracies, support for SMT solving, and interest in very large problems may require more fundamental revisions.

# Examples

- Find (a count of) all valuations of some collection of atoms

``` rust
// setup a context to solve within.
let mut the_context: Context = Context::from_config(Config::default(), None);

// Each character in the string is interpreted as an atom.
let atoms = "model";
for atom in atoms.chars() { // add atoms to the context.
    assert!(the_context.atom_from_string(&atom.to_string()).is_ok())
}

let mut count = 0;

loop {
    // Clear any decisions made on a previous solve and
    the_context.clear_decisions();
    // Determine the satisfiability of the formula in the context.
    assert!(the_context.solve().is_ok());

    // Break from the loop as soon as the context is unsatisfiable.
    match the_context.report() {
        report::Solve::Satisfiable => {}
        _ => break,
    };

    count += 1;

    // Read the (satisfying) valuation from the present solve.
    let valuation = the_context.atom_db.valuation_string();

    // Create the string representation of a clause to force a new valuation.
    let mut new_valuation = String::new();
    for literal in valuation.split_whitespace() {
        match literal.chars().next() {
            Some('-') => new_valuation.push_str(&literal[1..]),
            Some(_) => new_valuation.push_str(format!("-{literal}").as_str()),
            None => break,
        };
        new_valuation.push(' ');
    }

    // Transform the string to a clause and add the clause to the solve.
    let the_clause = the_context.clause_from_string(&new_valuation).unwrap();
    match the_context.add_clause(the_clause) {
        Ok(()) => {}
        Err(_) => break,
    };
}
// Check the expected number of models were found.
assert_eq!(count, 2_usize.pow(atoms.len().try_into().unwrap()));
```

- Parse and solve a DIMACS formula

``` rust
let mut the_context = Context::from_config(Config::default(), None);

let mut dimacs = vec![];
let _ = dimacs.write(b"
 p  q 0
-p  q 0
-p -q 0
 p -q 0
");

the_context.read_dimacs(dimacs.as_slice());
the_context.solve();
assert_eq!(the_context.report(), report::Solve::Unsatisfiable);
```

- Identify unsatisfiability of a DIMACS formula during parsing.

``` rust
let mut the_context = Context::from_config(Config::default(), None);

let mut dimacs = vec![];
let _ = dimacs.write(b"
 p       0
-p  q    0
-p -q  r 0
      -r 0
");

assert_eq!(the_context.read_dimacs(dimacs.as_slice()), Err(err::Build::Unsatisfiable));
```
