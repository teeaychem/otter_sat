/*!
C Bindings for the reentrant incremental sat solver API --- the IPASIR IPA.

Full bindings for the IPASIR API are given in [ipasir_one], and incomplete bindings for the IPASIR2 API may be found in [ipasir_two].

Information about the APIs may be found at:
- <https://github.com/biotomas/ipasir>, for IPASIR.
- <https://github.com/ipasir2/ipasir2>, for IPASIR2.

Note, 'solver' and 'context' are synonymous in this module.\
Though, strictly, 'solver' is only used as, or when referring to, the parameter of an API function, and 'context' is only used to refer to an instance of the context structure.

# Compiling a library

By default, cargo does not build a library suitable for to linking to a C program.\
For details on building a suitable library, see: <https://doc.rust-lang.org/reference/linkage.html>

# Efficiency

At present, the library uses a 'transparent' representation of literals added through the IPASIR API --- whether as part of a clause, as an assumption, etc.
This means if the literal -83 is added through the API all internal data structures will 'grow' to allow for 83 atoms.
In this respect, it is much more efficient to add a clause containing the largest literal first.

# Implementation details

## Bundles

For interaction with the API a context is bundled together with a few API specific structures in a [ContextBundle].
These structs are primarily used to buffer or cache information that a context has no general use for.

## Callbacks

Each IPASIR callback is implemented as a closure over some native callback.
*/

use std::{
    collections::{HashMap, HashSet},
    ffi::{c_int, c_void},
    sync::OnceLock,
};

use crate::{
    config::Config,
    context::Context,
    db::ClauseKey,
    structures::{
        atom::Atom,
        clause::{CClause, Clause, IntClause},
        literal::CLiteral,
    },
};

mod context_bundle;
pub use context_bundle::ContextBundle;

pub mod ipasir_one;
pub mod ipasir_two;

/// The signature of the solver, written (once) when needed using [env!].
pub static IPASIR_SIGNATURE: OnceLock<std::ffi::CString> = OnceLock::new();
