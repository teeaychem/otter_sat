use std::path::PathBuf;

use crossbeam::channel::Receiver;

use crate::{dispatch::Dispatch, frat};

pub fn frat_receiver(rx: Receiver<Dispatch>, frat_path: PathBuf) {
    let mut frat_writer = build_frat_writer(&Some(frat_path));

    while let Ok(dispatch) = rx.recv() {
        frat_writer(dispatch);
    }
}

/// If given a path the writer transcribes dispatches to the path as an FRAT proof.
/// Otherwise, then writer does nothing
pub fn build_frat_writer(frat_path: &Option<PathBuf>) -> Box<dyn FnMut(Dispatch)> {
    match frat_path {
        None => {
            let hand = |_: Dispatch| {};
            Box::new(hand)
        }
        Some(path) => {
            let mut transcriber = frat::Transcriber::new(path);
            let handler = move |dispatch: Dispatch| {
                transcriber.transcribe(dispatch);
                transcriber.flush()
            };
            Box::new(handler)
        }
    }
}
