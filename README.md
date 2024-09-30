Otter… other… odder… otter… a conflict-driven clause learning SAT solver, written for skill and research.

At present, features include:

- Clause learning thorugh analysis of implication graphs
- Clause forgetting based on glue principles (see: [Glucose](https://github.com/audemard/glucose) for details)
  - By default all clauses which do not match the required `glue` level are forgotten at regular intervals
- A [VSIDS](https://arxiv.org/abs/1506.08905) choice selection heuristic.
- Two-watch lazy inspection.
- Watch lists.
- An unsatisfiable core of the original formula, if the formua is UNSAT.
- Some documentation.
- logging via [log4rs](https://docs.rs/log4rs/latest/log4rs/) (see `config/log4rs.yaml`)
- A very long list of todos!

Arguments (with the help of [clap](https://docs.rs/clap/latest/clap/)):

| Long                  | Short | Use                                                                                                 |
|-----------------------|-------|-----------------------------------------------------------------------------------------------------|
| `--file`              | `-f`  | The formula to use, in [DIMACS CNF](https://jix.github.io/varisat/manual/0.2.0/formats/dimacs.html) |
| `--core`              | `-c`  | Display an unsatisfiable core on UNSAT                                                              |
| `--stats`             | `-s`  | Display some stats on SAT/UNSAT                                                                     |
| `--assignment`        | `-a`  | Display a satisfying assignment on SAT                                                              |
| `--glue-strength`     | `-g`  | Specify the lbd value required to retain a clause                                                   |
| `--stopping-criteria` |       | The stopping criteria to use (default: `FirstUIP`, alt: `None`)                                     |


Docmentation and tests are moslty added as the solver develops and parts solidify.
For the moment, things are fairly green.

Some resources I have found, or am finding, helpful:

| Resource                                                                                                                                | Author(s)                                                                   |
|-----------------------------------------------------------------------------------------------------------------------------------------|-----------------------------------------------------------------------------|
| [Handbook of Practical Logic and Automated Reasoning](https://doi.org/10.1017/CBO9780511576430)                                         | John Harrison                                                               |
| [Decision Procedures](https://doi.org/10.1007/978-3-662-50497-0)                                                                        | Daniel Kroening, Ofer Strichman                                             |
| [The Art of Computer Programming: Satisfiability](https://www-cs-faculty.stanford.edu/~knuth/taocp.html)                                | Donald E. Knuth                                                             |
| [Handbook of Satisfiability](https://www.iospress.com/catalog/books/handbook-of-satisfiability-2)                                       | Armin Biere, Marijn Heule, Hans van Maaren, Toby Walsh (eds.)               |
| [Understanding VSIDS Branching Heuristics in Conflict-Driven Clause-Learning SAT Solvers](https://doi.org/10.1007/978-3-319-26287-1_14) | Jia Hui Liang, Vijay Ganesh, Ed Zulkoski, Atulan Zaman, Krzysztof Czarnecki |
| [On the Glucose SAT Solver](https://doi.org/10.1142/S0218213018400018)                                                                  | Gilles Audemard, Laurent Simon                                              |
