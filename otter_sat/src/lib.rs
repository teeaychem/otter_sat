/*!
A library for determining the satisfiability of boolean formulas written in conjunctive normal form.

otter_sat is a library for determining the satisfiability of boolean formulas written in conjunctive normal form, using a variety of techniques from the literature on conflict-driven clause-learning solving, and with support for incremental solves.

otter_sat is developed to help researchers, developers, or anyone curious, to investigate satisfiability solvers, whether as a novice or through implementing novel ideas.
In this respect, otter_sat may (eventually) be seen as similar to [MiniSAT](minisat.se).

Some guiding principles of otter_sat are (see [below](#guiding-principles) for further details):
- [Modularity](#modularity).
- Documentation, of both implementation and theory.
- [Verification](#verification).
- [Simple efficiency](#simple-efficiency).

# Orientation

The library is design around the core structure of a [context].

Clauses may be added though the [DIMACS](crate::context::GenericContext::read_dimacs) representation of a formula or [programmatically](crate::context::GenericContext::add_clause).

Internally, and at a high-level, a solve is viewed in terms of manipulation of, and relationships between, a handful of databases which instantiate core theoretical objects.
Notably:
- A formula is stored in a clause database.
- A valuation is stored in an atom database.
- Consequences of the current valuation with respect to the formula are stored in a literal database.

Consequences follow a current valuation and formula, which in turn lead to a revised valuation and/or formula, from which further consequences follow.
And, in terms of implementation, data from the clause and atom database is read, used to update the literal database, which in turn leads to revisions of the atom and/or clause database.

Useful starting points, then, may be:
- The high-level [solve procedure](crate::procedures::solve) to inspect the dynamics of a solve.
- The [database module](crate::db) to inspect the data considered during a solve.
- The [structures] to familiarise yourself with the abstract elements of a solve and their representation (formulas, clauses, etc.)
- The [configuration](crate::config) to see what features are supported.

If you're in search of cnf formulas consider:
- The SATLIB benchmark problems at [www.cs.ubc.ca/~hoos/SATLIB/benchm.html](https://www.cs.ubc.ca/~hoos/SATLIB/benchm.html)
- The Global Benchmark Database at [benchmark-database.de](https://benchmark-database.de)
- SAT/SMT by Example at [smt.st](https://smt.st)

# Examples

+ Find a count of all valuations of some collection of atoms, while printing a representation of each model.

```rust
# use otter_sat::{
#    config::Config,
#    context::Context,
#    reports::Report,
#    structures::{
#        atom::Atom,
#        literal::{CLiteral, Literal},
#        valuation::Valuation,
#    },
# };
// The context in which a solve takes place.
let mut context: Context = Context::from_config(Config::default());

// Atoms will be represented by characters of some string.
let characters = "model".chars().collect::<Vec<_>>();
let mut atom_count: u32 = 0;

// Each call to fresh_atom expands the context to include a fresh (new) atom.
// Atoms form a contiguous range from 1 to some limit.
for _character in &characters {
    match context.fresh_atom() {
        Ok(_) => atom_count += 1,
        Err(_) => {
            panic!("Atom limit exhausted.")
        }
    }
}
let mut model_count = 0;

while let Ok(Report::Satisfiable) = context.solve() {
    model_count += 1;

    let mut valuation_representation = String::new();

    // To exclude the current valuation, the negation of the current valuation is added as a clause.
    // As valuations are conjunctions and clauses disjunctions, this may be done by negating each literal.
    let mut exclusion_clause = Vec::new();

    // The context provides an iterator over (atom, value) pairs.
    // Though every non-constant atom has a value in this model, this avoids handling the no value option.
    for (atom, value) in context.valuation().atom_valued_pairs() {
        // As atoms begin at 1, a step back is required to find the appropriate character.
        match value {
            true => valuation_representation.push(' '),
            false => valuation_representation.push('-'),
        }
        valuation_representation.push(characters[(atom as usize) - 1]);
        valuation_representation.push(' ');

        exclusion_clause.push(CLiteral::new(atom as Atom, !value));
    }

    valuation_representation.pop();
    println!("{model_count}\t {}", valuation_representation);

    // After a solve, the context is refreshed to clear any decisions made.
    // Learnt clauses remain, though any assumptions made are also removed.
    context.refresh();

    match context.add_clause(exclusion_clause) {
        Ok(_) => {}
        Err(_) => break,
    };
}

assert_eq!(model_count, 2_usize.pow(atom_count));
```

+ Parse and solve a DIMACS formula.

```rust
# use otter_sat::context::Context;
# use otter_sat::config::Config;
# use std::io::Write;
# use otter_sat::reports::Report;
# use otter_sat::types::err::{self};
let mut ctx = Context::from_config(Config::default());

let mut dimacs = vec![];
let _ = dimacs.write(b"
 1  2 0
-1  2 0
-1 -2 0
 1 -2 0
");

ctx.read_dimacs(dimacs.as_slice());
ctx.solve();
assert_eq!(ctx.report(), Report::Unsatisfiable);
```

# Guiding principles

## Modularity

  + A solver is built of many interconnected parts, but where possible (and reasonable) interaction between parts happens through documented access points. For example:
    - Clauses are stored in a [clause database](db::clause), and are accesseed through [keys](db::ClauseKey).
      An internal distinction is made between unit clauses, binary clauses, and long(er) clauses.
      This distinction is encoded in the clause keys, and supports a variety of methods, but the internal structure of the clause database is private.
    - Things such as [literals](structures::literal) and [clauses](structures::clause) are defined first as traits whose canonical instantiations are used only when there is 'good reason' to do so.
    - The algorithm for determining satisfiability is factored into a collection of [procedures].
    - Use of external crates is limited to crates which help support modularity, such as [log](https://docs.rs/log/latest/log/) and [rand](https://docs.rs/rand/latest/rand/).

## Verification

+ The core solver (excluding techniques such as subsumption) supports generation of [FRAT proofs](https://arxiv.org/pdf/2109.09665v1) which can be checked by independent tools such as [FRAT-rs](https://github.com/digama0/frat).

+ Verification itself is handled via a system for sending dispatches from a solve, and incurs minimal overhead (checks on an optional) when not used.\
  As a consequence of this, the system for handling dispatches is somewhat complex.
  And, as a consequence of *that* an effort has been made to make ignoring the system easy.

## Simple efficiency

The solver is efficient in most operations, and known inefficiencies are often noted.
Still, while compromises are made for the same of efficiency, overall the library is written using mostly simple Rust, with annotated uses of unsafe, notes on when using a function would be unsound, and fights with the borrow checker explained.
  + The library makes free use of unsafe so long as some reason is given for why safety is maintained.
  + Though, many relevant invariants escape the borrow checker, and for this purpose 'soundness' notes are made where relevant.
  + In addition, there are times when some not-so-simple Rust is required to appease the borrow checker (notably [BCP](crate::procedures::bcp)) and explanations are given of these.

# Logs

To help diagnose issues (somewhat) detailed calls to [log!](log) are made, and a variety of targets are defined in order to help narrow output to relevant parts of the library.
As logging is only built on request, and further can be requested by level, logs are verbose.

The targets are lists in [misc::log].

For example, when used with [env_logger](https://docs.rs/env_logger/latest/env_logger/):
- Logs related to [the clause database](crate::db::clause) can be filtered with `RUST_LOG=clause_db …` or,
- Logs of reduction count without information about the clauses removed can be found with `RUST_LOG=reduction=info …`

*/

#![allow(clippy::collapsible_else_if)]
#![allow(clippy::derivable_impls)]
#![allow(clippy::single_match)]
#![allow(mixed_script_confusables)]
#![allow(unused_labels)]
#![allow(unused_must_use)]
// #![allow(unused_imports)]
// #![warn(missing_docs)]

pub mod builder;
pub mod procedures;

pub mod config;
pub mod context;
pub mod structures;
pub mod types;

pub mod generic;

pub mod reports;

pub mod db;

pub mod atom_cells;
pub mod misc;

pub mod preprocessing;

#[allow(dead_code, unused_variables, unused_imports)]
pub mod ipasir;
