use std::path::PathBuf;

use crossbeam::channel::Receiver;

use crate::dispatch::{frat, Dispatch};

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
