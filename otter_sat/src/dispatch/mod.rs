/*!
Dispatches for external observers.

Dispatches have two uses:
- Communication after some procedure, e.g. a solve.
- Optional observation of the dynamics of a context and other related structures during some procedure.

Each dispatch is a small message of some pre-determined type, and may be sent through a callback.

- [library] contains all dispatch types, arranged in a fixed heirarchy.
- [frat] contains tools for creating FRAT proofs by using dispatches.
- [core] contains tools for identifying unsatisfiable cores by using dispatches.

Dispatches come in a variety of types;

- [Deltas](crate::dispatch::library::delta), on some change during a procedure or to an interal structure.
  - For example:
    - Addition and deletion of clauses.
    - The use of a clause when applying resolution to a conflict.
- [Reports](crate::dispatch::library::report), on the result of some procedure.
  - For example, whether a formula is satisfiable.
- [Stats](crate::dispatch::library::stat), regarding various things.
  - For example, the number of expected and processed clauses when importing a DIMACS formla.

Each type of dispatch has multiple subtypes.
If possible, these subtypes correspond to the procedure or structure from which the dispatch is sent.
For example, the `Delta` type of dispatch has deltas for boolean constraint propagation, the clause database, etc.

Dispatches are designed to be tidy to deconstruct by pattern matching, though as a consequence are somewhat messy to construct.
Further, as the name of a type of dispatch may conflict with, e.g., the name of the structure the dispatch is related to, dispatch creation is designed to be made relative to the module of the type of dispatch.

So, dispatches are typtically broken up into parts.
For example, the following dispatches information that a conflict was derived when applying bcp with `literal` freshly set due to the clause indexed by `clause_key`, with this dispatch itself being sent by passing it to the function `dispatcher`.

```ignore
let delta = delta::BCP::Conflict {
    from: literal,
    via: clause_key,
};
dispatcher(Dispatch::Delta(Delta::BCP(delta)));
```

After matching on the type `Delta` one may then match on `BCP`, and finally on `Conflict` to retreive this detail…

# Macros

The present implementation of dispatches is to check to see if a dispatcher is present, and if so to send a dispatch.
As such, the runtime overhead of unused dispatches is low (a check on an optional).
Still, these checks need to be written and due to the complexity of writing dispatches tend to distract from the non-dispatch code.

To help hide most of the complexity, macros are often used.

# Examples

Dispatching details on the addition of an original clause to the clause database.

The addition is communicated via a series of deltas, amounting to:
1. A signal that a sequence of deltas relating to the addition of a clause are to follow.
2. The details of some literal.
3. The type of clause and the key with which the clause was stored.

```ignore
use crate::dispatch::library::delta::{self};

if let Some(dispatcher) = &clause_db.dispatcher {
    let delta = delta::ClauseDB::ClauseStart;
    dispatcher(Dispatch::Delta(Delta::ClauseDB(delta)));
    for literal in &clause {
        let delta = delta::ClauseDB::ClauseLiteral(*literal);
        dispatcher(Dispatch::Delta(Delta::ClauseDB(delta)));
    }
    let delta = delta::ClauseDB::Original(the_key);
    dispatcher(Dispatch::Delta(Delta::ClauseDB(delta)));
}
```

Behaviour in line with these dispatches for a receiver may be to:
1. Set up a buffer to record details of a clause.
2. Record details of the literal to the buffer.
3. Finalise a record of the buffer using the metadata of it's source in the context and the key used for internal access.

*/

pub mod frat;

/// Dispatch types.
#[derive(Clone)]
pub enum Dispatch {
    /// A report. E.g. that a formula is unsatisfiable, that time has elapsed, or that the context contains *n* clauses.
    Report(Report),
}

/// Reports from the context.
#[derive(Clone)]
pub enum Report {
    /// Information regarding a solve.
    Solve(self::SolveReport),

    /// No further dispatches will be sent regarding the current solve.
    Finish,
}

/// High-level reports regarding a solve.
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum SolveReport {
    /// The formula of the context is satisfiable.
    Satisfiable,

    /// The formula of the context is unsatisfiable.
    Unsatisfiable,

    /// Satisfiability of the formula of the context could not be determined within the time allowed.
    TimeUp,

    /// Satisfiability of the formula of the context is unknown, for some reason.
    Unknown,
}

impl std::fmt::Display for self::SolveReport {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Satisfiable => write!(f, "Satisfiable"),
            Self::Unsatisfiable => write!(f, "Unsatisfiable"),
            Self::TimeUp => write!(f, "Unknown"),
            Self::Unknown => write!(f, "Unknown"),
        }
    }
}
