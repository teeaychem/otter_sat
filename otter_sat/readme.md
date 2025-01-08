# Overview

Otter… other… odder… otter_sat is a library for determining the satisfiability of boolean formulas written in conjunctive normal form.

otter_sat uses a variety of techniques from the literature on conflict-driven clause-learning solving, and with support for incremental solves.

otter_sat is developed to help researchers, developers, or anyone curious, to investigate satisfiability solvers, whether as a novice or through implementing novel ideas.

Some guiding principles of the library are:
- Modularity.
- Documentation, of both theory and implementation.
- Verification (of core parts of the library through the production of FRAT proofs and external proof checkers).
- Simple efficiency.

- For documentation, see [https://docs.rs/otter_sat](https://docs.rs/otter_sat).
- For an associated cli application and tools to verify (parts of) the library see [https://github.com/teeaychem/otter_sat](https://github.com/teeaychem/otter_sat).

# Use

- The documentation contains various examples, and a handful of example files are included in the crate (available via `cargo run --example sudoku`, etc.)
- [otter_tests](https://github.com/teeaychem/otter_sat/tree/main/otter_tests) is a crate to test the library against known problems, and to verify produced FRAT proofs and as such contains a variety of illustrative functions to help achieve these tasks.
- [otter_cli](https://github.com/teeaychem/otter_sat/tree/main/otter_cli) is a cli frontend to the library, which supports DIMACS encoded CNFs.

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
