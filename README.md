# 🦦 otter_sat

Otter… other… odder… a library for determining the satisfiability of boolean formulas written in conjunctive normal form, using a variety of techniques from the literature on conflict-driven clause-learning solving, and with support for incremental solves.

At present, this repository contains three key interrelated sub-directories:

- `otter_sat` the library, available as a crate from [crates.io](https://crates.io/crates/otter_sat) and with docs at [docs.rs/otter_sat](https://docs.rs/otter_sat).
  The library includes a (simple) cli interface.
- `otter_tests` a collection of tests to ensure the soundness of `otter_sat`, built as a crate.
- `otter_tests_plus` a collection of additional tests to ensure the soundness of `otter_sat`, which do not reasonably fit in a crate.
  For example, tests to ensure the library conforms to the IPASIR API.

# Features

At present, features include:

- Documentation of theory and implementation, some of which may be considered quite detailed --- see [docs.rs/otter_sat](https://docs.rs/otter_sat) (or via `cargo doc --lib`)
- Optional generation of [FRAT proofs](https://arxiv.org/pdf/2109.09665v1) which can be checked by independent tools such as [FRAT-rs](https://github.com/digama0/frat).
- Clause learning through analysis of implication graphs.
- Clause forgetting based on glue principles (see: [Glucose](https://github.com/audemard/glucose) for details).
  - By default all clauses which do not match the required `glue` level are forgotten at regular intervals.
- A [VSIDS](https://arxiv.org/abs/1506.08905) decision selection heuristic.
- Luby-based restarts.
- Watch literals and watch lists.
- Phase saving.
- Some optional unpredictability.
- On-the-fly self-subsumption.
- And more… (!)

# The CLI

The CLI is built as `otter_cli` in the target directory when compiling the library.

Alternatively, `cargo run` may be used.
For example:

``` shell
cargo run --profile release --features xz -- --model --atom_bump=1.3 --clause_decay=0.3 CNF_FILE.cnf.xz
```

# Examples

Some small examples of using the crate as a library are included in `examples/`.
Examples can be ran by passing the `example` flag and the name of the example to `cargo run`.
The general pattern is:

``` shell
cargo run [--profile <PROFILE>] --example <EXAMPLE NAME>
```

Specific examples are:

``` shell
cargo run --example sudoku
cargo run --example nonograms --profile release
cargo run --example all_models
```

Nonograms is reasonably quick in debug mode, but release is recommended.


# How this was made

I'm a sometimes logician, so the solver has been written in part to help understand modern SAT (research) and in part to improve the way I code (skill).
The methodology has been (and is) to read some theory, write some code, and do some thinking.
Some theory resources I have found, or am finding, helpful are noted through documentation and/or can be found in `resources/resources.bib`.

Solver resources used are:
- [kissat](https://github.com/arminbiere/kissat)
  For specific VSIDS details I could not find a paper for.
  Even more specifically, the limits and method for rescoring.
  This also led to FixedHeap structure.
