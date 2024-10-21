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

I'm a sometimes logician, so the solver has been written in part to help understand modern SAT (research) and in part to improve the way I code (skill).
The methodology has been (and is) to read some theory, write some code, and do some thinking.
Some resources I have found, or am finding, helpful can be found in `resources/resources.bib`
At some point I'll be happy with what's been implemented and look to contrast, when that happens I'll make a note of the solvers here.

Rust dependencies are:
- [log4rs](https://docs.rs/log4rs/latest/log4rs/)
  Logging (see `config/log4rs.yaml`)
- [clap](https://docs.rs/clap/latest/clap/)
  Configuration options
  - [clap-markdown](https://docs.rs/clap-markdown/latest/clap_markdown/)
    Markdown of the options, for below
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

**Usage:** `otter_sat [OPTIONS] --formula-file <FORMULA_FILE>`

###### **Options:**

* `-f`, `--formula-file <FORMULA_FILE>` — The DIMACS form CNF file to parse
* `-s`, `--stats` — Display stats on completion

  Default value: `false`
* `-v`, `--valuation` — Display a satisfying valuation, if possible

  Default value: `false`
* `-c`, `--core` — Display an unsatisfiable core on UNSAT

  Default value: `false`
* `-g`, `--glue-strength <GLUE_STRENGTH>` — Required glue strength

  Default value: `2`
* `--stopping-criteria <STOPPING_CRITERIA>` — Resolution stopping criteria

  Default value: `first-uip`

  Possible values:
  - `first-uip`:
    Resolve until the first unique implication point
  - `none`:
    Resolve on each clause used to derive the conflict

* `--VSIDS <VSIDS>` — Which VSIDS variant to use

  Default value: `mini-sat`

  Possible values:
  - `mini-sat`:
    Bump the activity of all variables in the a learnt clause
  - `chaff`:
    Bump the activity involved when using resolution to learn a clause

* `-r`, `--reduce-and-restart` — Reduce and restart, where:

  Default value: `false`
* `--reduce` — Allow for the clauses to be forgotten, on occassion

  Default value: `false`
* `--restart` — Allow for the decisions to be forgotten, on occassion

  Default value: `false`
* `--hobson` — Initially settle all atoms which occur with a unique polarity

  Default value: `false`
* `--random-choice-frequency <RANDOM_CHOICE_FREQUENCY>` — The chance of making a random choice (as opposed to using most VSIDS activity)

  Default value: `0`
* `-p`, `--polarity-lean <POLARITY_LEAN>` — The chance of choosing assigning positive polarity to a variant when making a choice

  Default value: `0`
* `-l`, `--luby <LUBY_U>` — The u value to use for the luby calculation when restarts are permitted

  Default value: `512`
* `-t`, `--time <TIME>` — Time limit for the solve
* `-u`, `--subsumption` — Allow (some simple) self-subsumption
I.e. when performing resolutinon some stronger form of a clause may be found
For example, p ∨ q ∨ r may be strengthened to p ∨ r
With subsumption the weaker clause is replaced (subsumed by) the stronger clause, this flag disables the process

  Default value: `false`
* `--tidy-watches` — Continue updating watches for all queued literals after a conflict

  Default value: `false`
