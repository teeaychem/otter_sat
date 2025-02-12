use std::path::PathBuf;

use otter_sat::dispatch::{
    frat::{self},
    Dispatch,
};

/// If given a path the writer transcribes dispatches to the path as an FRAT proof.
/// Otherwise, then writer does nothing
pub fn build_frat_writer(frat_path: Option<PathBuf>) -> Box<dyn FnMut(&Dispatch)> {
    let hand = |_: &Dispatch| {};
    Box::new(hand)
    // match frat_path {
    //     None => {
    //         let hand = |_: &Dispatch| {};
    //         Box::new(hand)
    //     }
    //     Some(path) => {
    //         let mut transcriber = frat::Transcriber::new(path).unwrap();
    //         let handler = move |dispatch: &Dispatch| {
    //             let _ = transcriber.transcribe(dispatch);
    //             transcriber.flush()
    //         };
    //         Box::new(handler)
    //     }
    // }
}
