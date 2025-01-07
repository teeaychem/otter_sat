# otter_cli

A cli interface to `otter_sat`, which features:

- A detailed `--help` overview of available features.
- Some numbers to look at while a solve happens.
- Optional generatation of an FRAT proof and/or unsatisfiable core, if the formua is UNSAT.
- xz support for easy [Global Benchmark Database](https://benchmark-database.de) interaction.

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
          Prevent decisions from being forgotten.

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

  -r, --random-decision-bias <BIAS>
          The chance of making a random decision (as opposed to using most VSIDS activity).

  -∠, --polarity-lean <LEAN>
          The chance of choosing assigning positive polarity to a variant when making a decision.

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
