# 🦦 otter_sat

Otter… other… odder… otter… a conflict-driven clause learning SAT solver, written for skill and research.

## Some notes…

At present, features include:

- Clause learning thorugh analysis of implication graphs
- Clause forgetting based on glue principles (see: [Glucose](https://github.com/audemard/glucose) for details)
  - By default all clauses which do not match the required `glue` level are forgotten at regular intervals
- A [VSIDS](https://arxiv.org/abs/1506.08905) choice selection heuristic.
- Luby-based restarts.
- Watch literals and watch lists.
- Phase saving.
- Some optional unpredictability.
- An unsatisfiable core of the original formula, if the formua is UNSAT.
- On-the-fly self-subsumption.
- Some documentation.
- xz support for easy [Global Benchmark Database](https://benchmark-database.de) interaction.

The command overview, below, may give some additional insight.

Docmentation and tests are moslty added as the solver develops and parts solidify.
For the moment, things are fairly green.

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

## Rust dependencies

Direct Rust dependencies are:
- [clap](https://docs.rs/clap/latest/clap/)
  Configuration options
- [crossterm](https://docs.rs/crossterm/latest/crossterm/)
  To dynamically display some stats during a solve in a static area of the terminal
- [xz2](https://docs.rs/xz2/)
  xz support
- [rand](https://docs.rs/rand/latest/rand/)
  To help make choices

Optional dependencies are:
- [log4rs](https://docs.rs/log4rs/latest/log4rs/)
  Logging (see `config/log4rs.yaml`)
- [tikv-jemallocator](https://github.com/marv/tikv-jemallocator)
  An alternative allocator

# Configuration

```
  -c, --show-core
          Display an unsatisfiable core on finding a given formula is unsatisfiable.

      --variable-decay <variable_decay>
          The decay to use for variable activity.
          Default: 50

          After a conflict any future variables will be bumped with activity (proportional to) 1 / (1 - decay^-3).
          Viewed otherwise, the activity of all variables is decayed by 1 - decay^-3 each conflict.
          For example, at decay of 3 at each conflict the activity of a variable decays to 0.875 of it's previous activity.

      --clause-decay <clause_decay>
          The decay to use for clause activity.
          Default: 20

          Works the same as variable activity, but applied to clauses.
          If reductions are allowed then clauses are removed from low to high activity.

      --reduction-interval <reduction-interval>
          The interval to perform reductions, relative to conflicts.
          Default: 500

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

      --no_subsumption
          Prevent (some simple) self-subsumption.

          That is, when performing resolutinon some stronger form of a clause may be found.
          Subsumption allows the weaker clause is replaced (subsumed by) the stronger clause.
          For example, p ∨ r subsumes p ∨ q ∨ r.

  -p, --preprocess
          Perform some pre-processing before a solve.
          For the moment this is limited to settling all atoms which occur with a unique polarity.

  -s, --stats
          Display stats during a solve.

  -v, --valuation
          Display valuation on completion.

  -g, --glue <STRENGTH>
          Required minimum (inintial) lbd to retain a clause during a reduction.
          Default: 3

  -🚏, --stopping-criteria <CRITERIA>
          The stopping criteria to use during resolution.
          Default: FirstUIP

            - FirstUIP: Resolve until the first unique implication point
            - None    : Resolve on each clause used to derive the conflict

  -🦇, --VSIDS <VARIANT>
          Which VSIDS variant to use.
          Default: MiniSAT

            - MiniSAT: Bump the activity of all variables in the a learnt clause.
            - Chaff  : Bump the activity involved when using resolution to learn a clause.

  -l, --luby <U>
          The 'u' value to use for the luby calculation when restarts are permitted.
          Default: 128

  -r, --random-choice-frequency <FREQUENCY>
          The chance of making a random choice (as opposed to using most VSIDS activity).
          Default: 0

  -∠, --polarity-lean <LEAN>
          The chance of choosing assigning positive polarity to a variant when making a choice.
          Default: 0

  -t, --time-limit <SECONDS>
          Time limit for the solve in seconds.
          Default: No limit
```
