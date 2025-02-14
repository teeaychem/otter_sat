/*!
Tools for building a context.

# Basic methods

The library has two basic methods for building a context:
- [fresh_atom](crate::context::GenericContext::fresh_atom), to obtain a fresh atom.
- [add_clause](crate::context::GenericContext::add_clause), to add a clause.

A formula may be added to a context by interweaving these two methods, together with relevant strucutre initialisers.
In rough strokes, the pattern is to:
- Obtain a collection of atoms to represent a clause.
- Create [CLiteral](crate::structures::literal::CLiteral)s from the atoms.
- Bundle the literals into a [CClause](crate::structures::clause::CClause).
- Add the clause to the context.

For examples, see below.
And, in particular, note this process may be simplified by using the canonical strucutres and associated methods.

# Examples

A clause built using basic methods.

```rust
# use otter_sat::context::Context;
# use otter_sat::config::Config;
# use otter_sat::reports::Report;
# use otter_sat::structures::{clause::CClause, literal::{CLiteral, Literal}};
#
let mut the_context = Context::from_config(Config::default());
let p = the_context.fresh_or_max_atom();
let q = the_context.fresh_or_max_atom();

let clause_a = CClause::from([CLiteral::new(p, true), CLiteral::new(q, false)]);
let clause_b = CClause::from([CLiteral::new(p, false), CLiteral::new(q, true)]);

 assert!(the_context.add_clause(clause_a).is_ok());
 assert!(the_context.add_clause(clause_b).is_ok());
 the_context.solve();
 assert_eq!(the_context.report(), Report::Satisfiable)
```

A simplified build, using canonical structures.

```rust
# use otter_sat::context::Context;
# use otter_sat::config::Config;
# use otter_sat::reports::Report;
# use otter_sat::structures::{clause::CClause, literal::{CLiteral, Literal}};
#
let mut the_context = Context::from_config(Config::default());
let p = the_context.fresh_or_max_literal();
let q = the_context.fresh_or_max_literal();

let clause_a = vec![p, -q];
let clause_b = vec![-p, q];

 assert!(the_context.add_clause(clause_a).is_ok());
 assert!(the_context.add_clause(clause_b).is_ok());
 the_context.solve();
 assert_eq!(the_context.report(), Report::Satisfiable)
```
*/
mod dimacs;
pub use dimacs::ParserInfo;

mod preprocess;
mod structures;

/// Ok results when adding a clause to the context.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ClauseOk {
    /// The clause was added to the context.
    Added,

    /// The clause was a tautology (and so was not added to the context).
    Tautology,
}
