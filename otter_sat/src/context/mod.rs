/*!
The context --- to which formulas are added and within which solves take place, etc.

Strictly, a [GenericContext] and a [Context].

The generic context is designed to be generic over various parameters.
Though, for the moment this is limited to the source of randomness.

Still, this helps distinguish generic context methods against those intended for external use or a particular application.
In particular, [from_config](Context::from_config) is implemented for a context rather than a generic context to avoid requiring a source of randomness to be supplied alongside a config.

# Example
```rust
# use otter_sat::context::Context;
# use otter_sat::config::Config;
# use otter_sat::reports::Report;
# use otter_sat::structures::literal::{CLiteral, Literal};
let mut the_context = Context::from_config(Config::default());

let p = the_context.fresh_or_max_atom();
let q = the_context.fresh_or_max_atom();

let p_q_clause = vec![CLiteral::new(p, true), CLiteral::new(q, true)];
assert!(the_context.add_clause(p_q_clause).is_ok());

let not_p = CLiteral::new(p, false);

assert!(the_context.add_clause(not_p).is_ok());
assert!(the_context.solve().is_ok());
assert_eq!(the_context.report(), Report::Satisfiable);

assert_eq!(the_context.atom_db.value_of(p), Some(false));
assert_eq!(the_context.atom_db.value_of(q), Some(true));
```
*/

pub mod callbacks;
mod counters;
pub use counters::Counters;
mod generic;
pub use generic::GenericContext;
mod specific;
pub use specific::Context;

use crate::db::ClauseKey;

#[allow(non_camel_case_types)]
#[derive(Debug, PartialEq, Eq)]
/// The state of a context.
/// These states correspond to the states defined in the IPASIR2 specification.
pub enum ContextState {
    /// The context allows for configuration.
    Configuration,

    /// The context allows input.
    Input,

    /// The database is known to be consistent, e.g. with a complete valuation.
    Satisfiable,

    /// The database is known to be inconsistnet, e.g. with an unsatisfiable clause identified.
    Unsatisfiable(ClauseKey),

    /// The consistency of the database is unknown.
    Solving,
}

impl std::fmt::Display for ContextState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Configuration => write!(f, "Configuration"),
            Self::Input => write!(f, "Input"),
            Self::Satisfiable => write!(f, "Satisfiable"),
            Self::Unsatisfiable(_) => write!(f, "Unsatisfiable"),
            Self::Solving => write!(f, "Solving"),
        }
    }
}
