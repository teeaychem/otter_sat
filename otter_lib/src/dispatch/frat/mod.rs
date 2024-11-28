//! Tools for creating FRAT proofs by using dispatches.
//!
//! Full specification of the FRAT format is documented in:
//! - *A Flexible Proof Format for SAT Solver-Elaborator Communication* (2022) Baek, Carneiro, and Heule.
//!   - [10.46298/lmcs-18(2:3)2022](https://doi.org/10.46298/lmcs-18(2:3)2022) ([arXiv](https://arxiv.org/abs/2109.09665v3) |  [LMCS](https://lmcs.episciences.org/9357))
//!
//! Steps:
//! - Original
//! - Addition
//! - Deletion
//! - Finalisation
//!
//! <div class="warning">
//!
//! - Transcription is sensitive to the order in which dispatches are received.
//!
//! - Transcription is not supported for
//! </div>

#[doc(hidden)]
pub mod transcriber;

use std::{collections::VecDeque, fs::File};

use crate::{db::keys::ClauseKey, structures::literal::Literal};

/// An intermediate struct to support transforming dispatches from a context to steps in an FRAT proof.
pub struct Transcriber {
    /// The file to which steps of the proof are written.
    file: File,

    /// A buffer holding steps until they are written to a file.
    step_buffer: Vec<String>,

    /// A buffer holding information about a clause
    clause_buffer: Vec<Literal>,

    /// A buffer holding information about clauses used during an instance of resolutions.
    resolution_buffer: Vec<ClauseKey>,

    variable_buffer: String,

    /// A queue of resolution buffers.
    resolution_queue: VecDeque<Vec<ClauseKey>>,

    /// A map from internal variables (the indicies) to external variables (the strings).
    variable_map: Vec<Option<String>>,
}
