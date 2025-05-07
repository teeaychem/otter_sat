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
- An 'xz' feature for reading compressed formulas from (e.g.) the [Global Benchmark Database](https://benchmark-database.de/).
- Clause learning through analysis of implication graphs.
- Clause forgetting based on glue principles (see: [Glucose](https://github.com/audemard/glucose) for details).
- A [VSIDS](https://arxiv.org/abs/1506.08905) decision selection heuristic.
- Luby-based restarts.
- Watch literals and watch lists.
- Phase saving.
- Some optional unpredictability.
- On-the-fly self-subsumption.
- Recursive clause minimisation.
- And more… (!)

# Verification

otter_sat supports verification of unsatisfiable report through optional generation of [FRAT proofs](https://arxiv.org/pdf/2109.09665v1).
FRAT proof may be checked by independent tools such as [FRAT-rs](https://github.com/digama0/frat).

Proofs are written by binding a writing structure to various callback functions exposed by otter_sat, with a reference implementation provided as part of [`otter_tests`](otter_tests/src/frat/mod.rs)

# Incremental solving and the IPASIR API

otter_sat provides full C bindings for the IPASIR API, and in turn supports incremental solving.
To use these bindings otter_sat should be compiled in a suitable way, e.g. as a cdylib.
For example, via:

```sh
cargo rustc --crate-type=cdylib
```

The setup and tests `ipasir_api` tests contained in `otter_tests_plus` may be used as reference material for compiling and linking to otter_sat as an external library.


# The CLI

A CLI to the solver is built as `otter_cli` in the target directory when compiling the library.

The CLI supports command supports adjustments to a all configuration options.
For details of the options, see the configuration module found at [otter_sat/src/config/mod.rs].

``` shell
otter_cli --model --atom_bump=1.3 --clause_decay=0.3 CNF_FILE.cnf
```

Alternatively, `cargo run` may be used.
For example:

``` shell
cargo run --profile release --features xz -- --model --atom_bump=1.3 --clause_decay=0.3 CNF_FILE.cnf.xz
```

# Examples

A minimal parse and solve is as follows:

``` rust
let mut dimacs = vec![];
let _ = dimacs.write(b"
 1  2 0
-1  2 0
-1 -2 0
 1 -2 0
");


let cfg = Config::default();
let mut ctx = Context::from_config(cfg);

ctx.read_dimacs(dimacs.as_slice());

ctx.solve();

assert_eq!(ctx.report(), Report::Unsatisfiable);
```

The core structure is a context, built using some configuration, and to which a formula is read, and a variety of methods associated with the context are called to determine information about the formula (though primarily whether the formula is satisfiable).

Some small examples of using the crate as a library are included in `examples/`.
Examples can be ran by passing the `example` flag and the name of the example to `cargo run`.
The general pattern is:

``` shell
cargo run [--profile <PROFILE>] --example <EXAMPLE NAME>
```

Specific examples are:

- Generation of all models of a formula:
  ``` shell
  cargo run --example all_models
  ```

- Reduction of sudoku to SAT:
  ``` shell
  cargo run --example sudoku
  ```

- Reduction of nonogram puzzles to SAT:
  ``` shell
  cargo run --example nonograms --profile release
  ```

Nonograms is reasonably quick in debug mode, but release is recommended.


# How otter_sat was made

I'm a sometimes logician, so the solver has been written in part to help understand modern SAT (research) and in part to improve the way I code (skill).
The methodology has been (and is) to read some theory, write some code, and do some thinking.
Some theory resources I have found, or am finding, helpful are noted through documentation.

As otter_sat has matured, insights from other solvers have been incorporated.
Similar to theory, the influence of other solvers is noted through documentation.
Though, as a top level resource, there is currently notable influence from:
- [kissat](https://github.com/arminbiere/kissat)
- [MiniSat](http://minisat.se/)
