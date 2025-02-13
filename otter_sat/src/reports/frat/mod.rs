/*!
Tools for creating FRAT proofs by using dispatches.

Full specification of the FRAT format is documented in:
- *A Flexible Proof Format for SAT Solver-Elaborator Communication* (2022) Baek, Carneiro, and Heule.
  - [10.46298/lmcs-18(2:3)2022](https://doi.org/10.46298/lmcs-18(2:3)2022) ([arXiv](https://arxiv.org/abs/2109.09665v3) |  [LMCS](https://lmcs.episciences.org/9357))

Steps:
- Original
- Addition
- Deletion
- Finalisation

<div class="warning">
- Transcription is not supported for solves which make use of subsumption.
  + More generally, unless noted it is safe to assume transcription is not supported for any solve which makes use of clause derivation/mutation techniques other than resolution.
</div>

# Use
Use by creating a listener for dispatches from a context and passing each dispatch to the transcriber.

# Notes

For the moment the transcriber automatically syncronises resolution information with new clauses by…
- Storing a clause after resolution has completed and before any other instance of resolution begins
  Specifically, the channel is FIFO and resolution information is stored in a FIFO queue.
  So, the contents of some buffered resolution information can always be associated with the relevant stored clause.

# Complications

A few decisions make this a little more delicate than it otherwise could be

- On-the-fly self-subsumption
  + For formulas, specifically,  means it's important to record an origial formula before subsumption is applied.
    Rather than do anything complex this is addressed by writing the original formula at the start of a proof.

- Atom renaming
  + … when mixed with 0 as a delimiter in the format requires (i think) translating a clause back to it's DIMACS representation
  - The context stores a translation, but to avoid interacting (and introducing mutexes) the transcriber listens for atoms being added to the context and keeps an internal map of their external string

- Multiple clause databases
  + Requires disambiguating indicies.
    As there are no explicit limits on indicies in the FRAT document, simple ASCII prefixes are used.
*/

#[doc(hidden)]
pub mod transcriber;

use std::{collections::VecDeque, fs::File};

use crate::db::ClauseKey;

/// An intermediate struct to support transforming dispatches from a context to steps in an FRAT proof.
pub struct Transcriber {
    /// The file to which steps of the proof are written.
    file: File,

    /// A buffer holding steps until they are written to a file.
    step_buffer: Vec<String>,

    /// A buffer holding information about clauses used during an instance of resolutions.
    resolution_buffer: Vec<ClauseKey>,

    /// A queue of resolution buffers.
    resolution_queue: VecDeque<Vec<ClauseKey>>,
}
