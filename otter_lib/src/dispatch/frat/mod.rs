//! Utilities for writing FRAT proofs
//!
//! <div class="warning">
//!
//! - Transcription is sensitive to the order in which dispatches are received.
//!
//! - Transcription is not supported for
//! </div>

/// Transcription
pub mod transcriber;

use std::{collections::VecDeque, fs::File, path::PathBuf};

use crossbeam::channel::Receiver;

use crate::{
    db::keys::ClauseKey,
    dispatch::{frat, Dispatch},
};

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

    /// A map from internal variables (the indicies) to external variables (the strings).
    variable_map: Vec<Option<String>>,
}

/// Passes dispatches on some channel to a writer for the given FRAT path until the channel closes.
pub fn frat_receiver(rx: Receiver<Dispatch>, frat_path: PathBuf) {
    let mut transcriber = frat::Transcriber::new(frat_path);
    let mut handler = move |dispatch: &Dispatch| {
        transcriber.transcribe(dispatch);
        transcriber.flush()
    };

    while let Ok(dispatch) = rx.recv() {
        handler(&dispatch);
    }
}
