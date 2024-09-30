A simple conflict driven SAT solver, written in Rust for skill and research.

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

| Long                | Short | Use                                                         |
| --file              | -f    | The formula to use, in DIMACS form                          |
| --core              | -c    | Display an unsatisfiable core on UNSAT                      |
| --stats             | -s    | Display some stats on SAT/UNSAT                             |
| --assignment        | -a    | Display a satisfying assignment on SAT                      |
| --glue-strength     | -g    | Specify the lbd value required to retain a clause           |
| --stopping-criteria |       | The stopping criteria to use (default: FirstUIP, alt: None) |

Docmentation and tests are moslty added as the solver develops and parts solidify.
For the moment, things are fairly green.
