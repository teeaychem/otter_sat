# 🦦 otter_sat

Otter… other… odder… a library for determining the satisfiability of boolean formulas written in conjunctive normal form, using a variety of techniques from the literature on conflict-driven clause-learning solving, and with support for incremental solves.

At present, this repository contains three interrelated crates:

- `otter_sat` the library.
- `otter_cli` a command line interface to `otter_sat`.
- `otter_tests` a collection of tests to ensure the soundness of `otter_sat`.

# Features

At present, features include:

- Documentation of theory and implementation, some of which may be considered quite detailed.
- Optional generation of [FRAT proofs](https://arxiv.org/pdf/2109.09665v1) which can be checked by independent tools such as [FRAT-rs](https://github.com/digama0/frat).
- Clause learning thorugh analysis of implication graphs.
- Clause forgetting based on glue principles (see: [Glucose](https://github.com/audemard/glucose) for details).
  - By default all clauses which do not match the required `glue` level are forgotten at regular intervals.
- A [VSIDS](https://arxiv.org/abs/1506.08905) decision selection heuristic.
- Luby-based restarts.
- Watch literals and watch lists.
- Phase saving.
- Some optional unpredictability.
- On-the-fly self-subsumption.

# Examples

Some small examples of using the crate as a library are included in `examples/`.
Examples can be ran by passing the `example` flag and the name of the example to `cargo run`.
The general pattern is:

```
cargo run --example <EXAMPLE NAME>
```

Specific examples are:

```
cargo run --example sudoku
cargo run --example all_models
```


# How this was made

I'm a sometimes logician, so the solver has been written in part to help understand modern SAT (research) and in part to improve the way I code (skill).
The methodology has been (and is) to read some theory, write some code, and do some thinking.
Some theory resources I have found, or am finding, helpful are noted through documentation and/or can be found in `resources/resources.bib`.

Solver resources used are:
- [kissat](https://github.com/arminbiere/kissat)
  For specific VSIDS details I could not find a paper for.
  Even more specifically, the limits and method for rescoring.
  This also led to FixedHeap structure.
