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
- Very basic clause subsumption.
- Some documentation.
- Various todos!

The command overview, below, may give some additional insight.

Docmentation and tests are moslty added as the solver develops and parts solidify.
For the moment, things are fairly green.

### How this was made

I'm a sometimes logician, so the solver has been written in part to help understand modern SAT (research) and in part to improve the way I code (skill).
The methodology has been (and is) to read some theory, write some code, and do some thinking.
Some resources I have found, or am finding, helpful can be found in `resources/resources.bib`.
At some point I'll be happy with what's been implemented and look to contrast, when that happens I'll make a note of the solvers here.

## Rust dependencies

Direct Rust dependencies are:
- [log4rs](https://docs.rs/log4rs/latest/log4rs/)
  Logging (see `config/log4rs.yaml`)
- [clap](https://docs.rs/clap/latest/clap/)
  Configuration options
- [petgraph](https://docs.rs/petgraph/latest/petgraph/)
  To help recod resolution history
- [slotmap](https://docs.rs/slotmap/latest/slotmap/)
  To help forget clauses
- [crossterm](https://docs.rs/crossterm/latest/crossterm/)
  To dynamically display some stats during a solve in a static area of the terminal
- [tikv-jemallocator](https://github.com/marv/tikv-jemallocator)
  An alternative allocator, optional
- [rand](https://docs.rs/rand/latest/rand/)
  Perhaps

# Configuration

```
Usage: otter_sat [OPTIONS] [paths]...

Arguments:
  [paths]...
          The DIMACS form CNF files to parse.

Options:
  -c, --show-core
          Display an unsatisfiable core on finding a given formula is unsatisfiable.

      --no-reduction
          Prevent clauses from being forgotten.

      --no-restart
          Prevent decisions being forgotten.

      --persevere
          Deny both to reduce and to restart.
          Equivalent to passing both '--no-reduction' and 'no_restarts'.

  -p, --preprocess
          Perform some pre-processing before a solve.
          For the moment this is limited to settling all atoms which occur with a unique polarity.

  -s, --stats
          Display stats during a solve.

  -u, --subsumption
          Allow (some simple) self-subsumption.

          That is, when performing resolutinon some stronger form of a clause may be found.
          Subsumption allows the weaker clause is replaced (subsumed by) the stronger clause.
          For example, p ∨ r subsumes p ∨ q ∨ r.

      --tidy-watches
          Continue updating watches for all queued literals after a conflict.

  -v, --valuation
          Display valuation on completion.

  -g, --glue <STRENGTH>
          Required minimum (inintial) lbd to retain a clause during a reduction.
          Default: 2

      --stopping-criteria <CRITERIA>
          The stopping criteria to use during resolution.
          Default: FirstUIP

            - FirstUIP: Resolve until the first unique implication point
            - None    : Resolve on each clause used to derive the conflict

          [possible values: first-uip, none]

      --VSIDS-variant <VARIANT>
          Which VSIDS variant to use.
          Default: MiniSAT

            - MiniSAT: Bump the activity of all variables in the a learnt clause.
            - Chaff  : Bump the activity involved when using resolution to learn a clause.

          [possible values: mini-sat, chaff]

  -l, --luby <U>
          The 'u' value to use for the luby calculation when restarts are permitted.
          Default: 512

  -r, --random-choice-frequency <FREQUENCY>
          The chance of making a random choice (as opposed to using most VSIDS activity).
          Default: 0

      --polarity-lean <LEAN>
          The chance of choosing assigning positive polarity to a variant when making a choice.
          Default: 0

  -t, --time-limit <LIMIT>
          Time limit for the solve in seconds.
          Default: No limit
```
