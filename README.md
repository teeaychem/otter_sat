# 🦦 otter_sat

Otter… other… odder… a SAT library, written for skill and research.

At present, this repository contains three interrelated crates:

- `otter_lib` a library for conflict-driven clause learning SAT solving.
- `otter_cli` a command line interface to `otter_lib`.
- `otter_tests` a collection of tests to ensure the soundness of `otter_lib`.

## Features

`otter_lib` is designed as to be a readable and extensible library.

Emphasis is placed on sensible abstractions and modularity, with the caveat that what makes an abstraction 'sensible' varies from being to being.
In any case, most parts of the library are (to be) supported by detailed documentation and examples.

### otter_lib

At present, features include:

- Optional generation of [FRAT proofs](https://arxiv.org/pdf/2109.09665v1) which can be checked by independent tools such as [FRAT-rs](https://github.com/digama0/frat).
- Clause learning thorugh analysis of implication graphs.
- Clause forgetting based on glue principles (see: [Glucose](https://github.com/audemard/glucose) for details).
  - By default all clauses which do not match the required `glue` level are forgotten at regular intervals.
- A [VSIDS](https://arxiv.org/abs/1506.08905) choice selection heuristic.
- Luby-based restarts.
- Watch literals and watch lists.
- Phase saving.
- Some optional unpredictability.
- On-the-fly self-subsumption.

### otter_cli

- A detailed `--help` overview of available features.
- Some numbers to look at while a solve happens.
- Optional generatation of an FRAT proof and/or unsatisfiable core, if the formua is UNSAT.
- xz support for easy [Global Benchmark Database](https://benchmark-database.de) interaction.


## Examples

Some small examples of using the crate as a library are included in `examples/`.
Examples can be ran by passing the `example` flag and the name of the example to `cargo run`.
The general pattern is:

```
cargo run --example <EXAMPLE NAME>
```

Specific examples are:

```
cargo run --example
cargo run --example all_models
```


### How this was made

I'm a sometimes logician, so the solver has been written in part to help understand modern SAT (research) and in part to improve the way I code (skill).
The methodology has been (and is) to read some theory, write some code, and do some thinking.
Some theory resources I have found, or am finding, helpful can be found in `resources/resources.bib`.

Solver resources used are:
- [kissat](https://github.com/arminbiere/kissat)
  For specific VSIDS details I could not find a paper for.
  Even more specifically, the limits and method for rescoring.
  This also led to FixedHeap structure.

# cli configuration

```
  -c, --show-core
          Display an unsatisfiable core on finding a given formula is unsatisfiable.

      --variable-decay <variable_decay>
          The decay to use for variable activity.

          After a conflict any future variables will be bumped with activity (proportional to) 1 / (1 - decay^-3).
          Viewed otherwise, the activity of all variables is decayed by 1 - decay^-3 each conflict.
          For example, at decay of 3 at each conflict the activity of a variable decays to 0.875 of it's previous activity.

      --clause-decay <clause_decay>
          The decay to use for clause activity.

          Works the same as variable activity, but applied to clauses.
          If reductions are allowed then clauses are removed from low to high activity.

      --reduction-interval <reduction_interval>
          The interval to perform reductions, relative to conflicts.

          After interval number of conflicts the clause database is reduced.
          Clauses of length two are never removed.
          Clauses with length greater than two are removed, low activity to high (and high lbd to low on activity ties).

      --no-reduction
          Prevent clauses from being forgotten.

      --no-restart
          Prevent choices from being forgotten.

  -🐘, --elephant
          Remember everything.
          Equivalent to passing both '--no-reduction' and 'no_restarts'.

      --no-subsumption
          Prevent (some simple) self-subsumption.

          That is, when performing resolutinon some stronger form of a clause may be found.
          Subsumption allows the weaker clause is replaced (subsumed by) the stronger clause.
          For example, p ∨ r subsumes p ∨ q ∨ r.

  -p, --preprocess
          Perform some pre-processing before a solve.
          For the moment this is limited to settling all atoms which occur with a unique polarity.

  -g, --glue <STRENGTH>
          Required minimum (inintial) lbd to retain a clause during a reduction.

  -🚏, --stopping-criteria <CRITERIA>
          The stopping criteria to use during resolution.

            - FirstUIP: Resolve until the first unique implication point
            - None    : Resolve on each clause used to derive the conflict

  -🦇, --VSIDS <VARIANT>
          Which VSIDS variant to use.

            - MiniSAT: Bump the activity of all variables in the a learnt clause.
            - Chaff  : Bump the activity involved when using resolution to learn a clause.

  -l, --luby <U>
          The 'u' value to use for the luby calculation when restarts are permitted.

  -r, --random-choice-frequency <FREQUENCY>
          The chance of making a random choice (as opposed to using most VSIDS activity).

  -∠, --polarity-lean <LEAN>
          The chance of choosing assigning positive polarity to a variant when making a choice.

  -t, --time-limit <SECONDS>
          Time limit for the solve in seconds.
          Default: No limit

  -d, --detail <LEVEL>
          The level to which details are communicated during a solve.
          Default: 0

  -s, --stats
          Display stats during a solve.

  -v, --valuation
          Display valuation on completion.

      --FRAT
          Write an FRAT proof.

      --FRAT-path <PATH>
          The path to write an FRAT proof.

  -h, --help
          Print help (see a summary with '-h')

  -V, --version
          Print version
```
